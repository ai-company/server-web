use actix_web::cookie::Cookie;
use actix_web::http::header;
use actix_web::web;
use actix_web::{web::Data, HttpResponse, Responder};

use handlebars::Handlebars;
use time::Duration;

use serde::Deserialize;

use crate::database::{token, user, SqlPool};
use crate::route::{EndpointProcessingError, EndpointProcessingResult};
use token::Token;
use user::User;

#[derive(Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

async fn signin(req: SignInRequest, pool: &SqlPool) -> EndpointProcessingResult<Token> {
    let User {
        password: stored_hash,
        id: user_id,
        ..
    } = match user::get(&req.email, pool)? {
        Some(v) => v,
        None => return Err(EndpointProcessingError::Unauthorized),
    };

    if bcrypt::verify(&req.password, &stored_hash).unwrap() {
        Ok(token::generate(pool, user_id)?)
    } else {
        Err(EndpointProcessingError::Unauthorized)
    }
}

// Wrapper

pub async fn get(template: Data<Handlebars<'_>>) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/user/signin", &()).unwrap())
}

pub async fn post(req: web::Form<SignInRequest>, pool: web::Data<SqlPool>) -> impl Responder {
    use EndpointProcessingError::*;

    match signin(req.into_inner(), pool.as_ref()).await {
        Ok(token) => HttpResponse::Found()
            .cookie(
                Cookie::build("auth", token)
                    .path("/")
                    .http_only(true)
                    .max_age(Duration::days(30))
                    .finish(),
            )
            .set_header(header::LOCATION, "/user/account")
            .finish(),
        Err(Unauthorized) => HttpResponse::Unauthorized()
            .header(header::LOCATION, "/")
            .finish(),
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
