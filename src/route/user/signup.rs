use actix_web::{
    cookie::Cookie,
    http::header,
    web::{self, Data},
    HttpResponse, Responder,
};
use serde::Deserialize;

use handlebars::Handlebars;
use time::Duration;

use crate::{
    database::{
        token::{self, Token},
        user, SqlPool,
    },
    route::EndpointProcessingResult,
};

#[derive(Deserialize)]
pub struct UserCreationRequest {
    pub email: String,
    pub password: String,
}

async fn signup(req: UserCreationRequest, pool: &SqlPool) -> EndpointProcessingResult<Token> {
    user::insert(
        &req.email,
        &bcrypt::hash(&req.password, bcrypt::DEFAULT_COST).unwrap(),
        pool,
    )?;

    let user = user::get(&req.email, pool)?.unwrap();

    Ok(token::generate(pool, user.id)?)
}

// wrappers

pub async fn get(template: Data<Handlebars<'_>>) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/user/signup", &()).unwrap())
}

pub async fn post(req: web::Form<UserCreationRequest>, pool: web::Data<SqlPool>) -> impl Responder {
    match signup(req.into_inner(), pool.as_ref()).await {
        Ok(token) => HttpResponse::Found()
            .cookie(
                Cookie::build("auth", token)
                    .path("/")
                    .http_only(true)
                    .max_age(Duration::days(30))
                    .finish(),
            )
            .header(header::LOCATION, "/user/account")
            .finish(),
        Err(e) => {
            println!("Failed to create user with error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
