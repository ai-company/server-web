use actix_web::web;
use serde_json::json;
use time::Duration;
use user::User;

use crate::middleware;
use handlebars::Handlebars;
use serde::Deserialize;

use sessions::SessionToken;

use crate::database::{sessions, user, SqlPool};
use crate::route::{EndpointProcessingError, EndpointProcessingResult};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header;
use actix_web::{web::Data, HttpResponse, Responder};

#[derive(Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

async fn signin(req: SignInRequest, pool: &SqlPool) -> EndpointProcessingResult<SessionToken> {
    let User {
        password: stored_hash,
        id: user_id,
        ..
    } = match user::get(&req.email, pool)? {
        Some(v) if v.verified => v,
        _ => return Err(EndpointProcessingError::Unauthorized),
    };

    if bcrypt::verify(&req.password, &stored_hash).unwrap() {
        Ok(sessions::generate(pool, user_id)?)
    } else {
        Err(EndpointProcessingError::Unauthorized)
    }
}

// Wrapper

pub async fn get(template: Data<Handlebars<'_>>, session: middleware::Session) -> impl Responder {
    if let middleware::Session::Guest = session {
        HttpResponse::Ok().body(template.render("page/user/signin", &()).unwrap())
    } else {
        HttpResponse::Found()
            .header(header::LOCATION, "/user/account")
            .finish()
    }
}

pub async fn post(
    req: web::Form<SignInRequest>,
    pool: web::Data<SqlPool>,
    template: Data<Handlebars<'_>>,
) -> impl Responder {
    use EndpointProcessingError::*;

    match signin(req.into_inner(), pool.as_ref()).await {
        Ok(token) => HttpResponse::Found()
            .cookie(
                Cookie::build("session", token)
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .max_age(Duration::days(30))
                    .finish(),
            )
            .set_header(header::LOCATION, "/user/account")
            .finish(),
        Err(Unauthorized) => HttpResponse::Unauthorized()
            .header(header::LOCATION, "/")
            .body(
                template
                    .render(
                        "page/user/signin",
                        &json!({
                            "validation": {
                                "email": ["Email or password is incorrect"]
                            }
                        }),
                    )
                    .unwrap(),
            ),
        Err(RequestUnparsable) | Err(RequestNonsensical) => HttpResponse::BadRequest()
            .header(header::LOCATION, "/")
            .finish(),
        Err(e) => {
            println!("Failed to create user with error: {}", e);
            HttpResponse::InternalServerError()
                .header(header::LOCATION, "/")
                .finish()
        }
    }
}
