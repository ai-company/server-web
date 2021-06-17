use actix_web::{
    http::header,
    web::{self, Data},
    HttpResponse, Responder,
};
use serde::Deserialize;

use handlebars::Handlebars;

use crate::{
    database::{user, SqlPool},
    route::EndpointProcessingResult,
};

#[derive(Deserialize)]
pub struct UserCreationRequest {
    pub email: String,
    pub password: String,
}

async fn signup(req: UserCreationRequest, pool: &SqlPool) -> EndpointProcessingResult<()> {
    user::insert(
        &req.email,
        &bcrypt::hash(&req.password, bcrypt::DEFAULT_COST).unwrap(),
        pool,
    )?;


    Ok(())
}

// wrappers

pub async fn get(template: Data<Handlebars<'_>>) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/user/signup", &()).unwrap())
}

pub async fn post(req: web::Form<UserCreationRequest>, pool: web::Data<SqlPool>) -> impl Responder {
    match signup(req.into_inner(), pool.as_ref()).await {
        Ok(_) => HttpResponse::Created()
            .header(header::LOCATION, "/user/account")
            .finish(),
        Err(e) => {
            println!("Failed to create user with error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
