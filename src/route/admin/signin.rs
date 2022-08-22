use actix_web::web;
use serde_json::json;
use time::Duration;
use user::User;

use crate::database::admin::{self, Admin};
use crate::middleware;
use handlebars::Handlebars;
use serde::Deserialize;

use sessions::SessionToken;

use crate::database::{admin_sessions, sessions, user, SqlPool};
use crate::route::{EndpointProcessingError, EndpointProcessingResult};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header;
use actix_web::{web::Data, HttpResponse, Responder};

#[derive(Deserialize)]
pub struct SignInRequest {
    pub username: String,
    pub password: String,
}

async fn signin(req: SignInRequest, pool: &SqlPool) -> EndpointProcessingResult<SessionToken> {
    let Admin {
        password: stored_hash,
        id: user_id,
        ..
    } = match admin::get(&req.username, pool)? {
        Some(v) => v,
        _ => return Err(EndpointProcessingError::Unauthorized),
    };

    if bcrypt::verify(&req.password, &stored_hash).unwrap() {
        Ok(admin_sessions::generate(pool, user_id)?)
    } else {
        Err(EndpointProcessingError::Unauthorized)
    }
}

// Wrapper

pub async fn get(
    template: Data<Handlebars<'_>>,
    session: middleware::context::SessionContext,
) -> impl Responder {
    if let middleware::AdminSession::Guest = session.admin {
        HttpResponse::Ok().body(template.render("page/admin/signin", &session).unwrap())
    } else {
        HttpResponse::Found()
            .header(header::LOCATION, "/admin/account")
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
                Cookie::build("admin", token)
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .max_age(Duration::days(1))
                    .finish(),
            )
            .set_header(header::LOCATION, "/admin/account")
            .finish(),
        Err(Unauthorized) => HttpResponse::Unauthorized()
            .header(header::LOCATION, "/")
            .body(
                template
                    .render(
                        "page/admin/signin",
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
            println!("Failed to create admin with error: {}", e);
            HttpResponse::InternalServerError()
                .header(header::LOCATION, "/")
                .finish()
        }
    }
}
