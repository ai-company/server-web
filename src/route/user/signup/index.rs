use std::{collections::HashMap, fmt::Debug};

use actix_web::{
    cookie::{Cookie, SameSite},
    http::header,
    web::{self, Data},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};

use serde::{Deserialize, Serialize};
use strum::EnumVariantNames;

use handlebars::Handlebars;
use serde_json::json;
use time::Duration;

use crate::{
    database::{billing_info, sessions::SessionToken, signup_tokens, user, SqlPool},
    emails::{account::confirmation::AccountConfirmationEmail, Email},
    middleware,
    route::EndpointProcessingError,
    util::FormData,
    validator::{v, Validator, ValidatorResult},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreationRequest {
    pub email: String,
    pub password: String,
    pub tier: String,
    pub fullname: String,
    pub cvr: Option<String>,
    pub studentid: Option<Vec<u8>>,
    pub address: String,
    pub city: String,
    pub zip: String,
    pub country: String,
    pub terms: Option<String>,
}

#[derive(Debug)]
pub enum UserCreationError {
    EndpointError(EndpointProcessingError),
    FormError(ValidatorResult),
    InternalError(String),
    UserAlreadyExists,
}

impl std::error::Error for UserCreationError {}
impl std::fmt::Display for UserCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UserCreationError::*;
        match self {
            FormError(e) => write!(f, "Form error: {:?}", e),
            EndpointError(e) => write!(f, "{}", e),
            InternalError(e) => write!(f, "Internal Error: {}", e),
            UserAlreadyExists => write!(f, "User already exists"),
        }
    }
}

impl From<EndpointProcessingError> for UserCreationError {
    fn from(e: EndpointProcessingError) -> Self {
        UserCreationError::EndpointError(e)
    }
}

impl From<Box<dyn std::error::Error>> for UserCreationError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        UserCreationError::EndpointError(EndpointProcessingError::Processing(e))
    }
}

pub type UserCreationResult<T> = Result<T, UserCreationError>;

#[derive(Debug, EnumVariantNames)]
#[allow(non_camel_case_types, dead_code)]
pub enum Tier {
    student,
    private,
    business,
}

fn signup_validate_form(req: &FormData) -> ValidatorResult {
    let mut validator = Validator::new();

    validator
        .field("email", &[v::required, v::email])
        .field("password", &[v::required, v::length::<8, 0>])
        .field("tier", &[v::required, v::one_of::<Tier>])
        .field("fullname", &[v::required])
        .field("address", &[v::required])
        .field("city", &[v::required])
        .field("zip", &[v::required])
        .field("country", &[v::required])
        .field("terms", &[v::required]);

    match req.get("tier").and_then(|t| t.string()) {
        Some("student") => validator.field(
            "studentid",
            &[v::required, v::file_size::<0, { v::MiB(5) }>],
        ),
        Some("business") => validator.field("cvr", &[v::required]),
        _ => &validator,
    };

    let req = req.into();

    validator.check(&req)
}

/// Signup flow:
/// - if user exists:
///     - if signup token exists and not expired:
///         - resend email
///     - else if signup token exists and expired:
///         - generate new signup token and send email
/// - else:
///     - generate signup token and send email
///
async fn signup(
    data: &FormData,
    pool: &SqlPool,
    handlebars: &Handlebars<'_>,
) -> UserCreationResult<SessionToken> {
    use UserCreationError::*;

    let validation = signup_validate_form(data);
    if !validation.is_empty() {
        return Err(FormError(validation));
    }

    let mut req = match data.parse::<UserCreationRequest>() {
        Ok(r) => r,
        Err(e) => return Err(InternalError(format!("Form processing failure: {}", e))),
    };

    if user::get(&req.email, pool)?.is_some() {
        // let mut errmap = HashMap::new();
        // errmap.insert(
        //     "email".into(),
        //     vec!["an account with this e-mail already exists".to_owned()],
        // );
        // return Err(FormError(errmap));
        return Err(UserCreationError::UserAlreadyExists);
    }

    req.password = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST).unwrap();

    user::insert(&req, pool)?;
    let user = user::get(&req.email, pool)?.unwrap();

    billing_info::insert(&req, &user, pool)?;

    let (token, session_key) = signup_tokens::generate(pool, user.id)?;

    match AccountConfirmationEmail::new(&token)
        .render(handlebars)
        .send(&req.email)
    {
        Ok(_) => Ok(session_key),
        Err(_) => Err(InternalError("Failed to send email".to_string())),
    }
}

// wrappers

#[derive(Debug, Deserialize, Serialize)]
pub struct SignupTierQuery {
    pub tier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResendQuery {
    resend: Option<String>,
}

/// Get current `session` cookie if one exists
pub fn get_signup_key(req: &HttpRequest) -> Option<String> {
    req.cookies()
        .map(|jar| {
            jar.iter()
                .find(|cookie| cookie.name() == "signup_session_key")
                .map(|auth| auth.value().into())
        })
        .unwrap_or(None)
}

pub async fn get(
    req: HttpRequest,
    template: Data<Handlebars<'_>>,
    session: middleware::Session,
    tier: web::Query<SignupTierQuery>,
    resend: web::Query<ResendQuery>,
    pool: web::Data<SqlPool>,
) -> impl Responder {
    if resend.resend.is_some() {
        if let Some(signup_key) = get_signup_key(&req) {
            let token = signup_tokens::get_by_signup_key(&pool, &signup_key)
                .unwrap()
                .unwrap();
            let user_id = signup_tokens::get_owner(&pool, &token).unwrap().unwrap();
            let user = user::get_with_id(user_id, &pool).unwrap().unwrap();

            return match AccountConfirmationEmail::new(&token)
                .render(&template)
                .send(&user.email)
            {
                Ok(_) => HttpResponse::Ok()
                    .body(template.render("page/user/signup/continue", &()).unwrap()),
                Err(_) => {
                    HttpResponse::InternalServerError().body("Failed to send email".to_string())
                }
            };
        }
    }

    if let middleware::Session::Guest = session {
        HttpResponse::Ok().body(
            template
                .render("page/user/signup", &json!({"tier": tier.into_inner()}))
                .unwrap(),
        )
    } else {
        HttpResponse::Found()
            .header(header::LOCATION, "/user/account")
            .finish()
    }
}

pub async fn post(
    req: FormData,
    pool: web::Data<SqlPool>,
    template: Data<Handlebars<'_>>,
    tier: web::Query<SignupTierQuery>,
) -> impl Responder {
    match signup(&req, pool.as_ref(), &template).await {
        Ok(session_key) => HttpResponse::Ok()
            .cookie(
                Cookie::build("signup_session_key", session_key)
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .max_age(Duration::hour())
                    .finish(),
            )
            // .header(
            //     header::LOCATION,
            //     format!(
            //         "/user/signup/payment?tier={}",
            //         req.get("tier").and_then(|t| t.string()).unwrap()
            //     ),
            // )
            // .finish(),
            .body(template.render("page/user/signup/continue", &()).unwrap()),
        Err(UserCreationError::UserAlreadyExists) => {
            println!("[siup] user already exists");
            HttpResponse::Ok()
                .cookie(
                    Cookie::build("signup_session_key", signup_tokens::generate_signup_key())
                        .path("/")
                        .http_only(true)
                        .same_site(SameSite::Strict)
                        .max_age(Duration::hour())
                        .finish(),
                )
                .body(template.render("page/user/signup/continue", &()).unwrap())
        }
        Err(UserCreationError::FormError(e)) => HttpResponse::BadRequest().body(
            template
                .render(
                    "page/user/signup",
                    &json!({
                        "tier": tier.into_inner(),
                        "validation": e,
                        "old": req.iter().map(|(k, v)| (k, v.string())).collect::<HashMap<_, _>>(),
                    }),
                )
                .unwrap(),
        ),
        Err(e) => {
            println!("Failed to create user with error: {}", e);
            HttpResponse::InternalServerError().body(template.render("page/user/signup", &json!({
                "tier": tier.into_inner(),
                "validation": {
                    "email": ["An unexpected error happened while signing-up. Please try again later"]
                },
                "old": req.iter().map(|(k, v)| (k, v.string())).collect::<HashMap<_, _>>()
            })).unwrap())
        }
    }
}
