use actix_web::{
    cookie::Cookie,
    web::{self, Query},
    HttpRequest, HttpResponse, Responder,
};
use handlebars::Handlebars;
use serde::Deserialize;
use time::Duration;

use crate::{middleware, tokens::sell_token};

#[derive(Deserialize)]
pub struct TokenQuery {
    token: Option<String>,
}

pub async fn get(
    req: HttpRequest,
    query: Query<TokenQuery>,
    template: web::Data<Handlebars<'_>>,
) -> impl Responder {
    if middleware::is_authorized(&req) {
        HttpResponse::Ok().body(template.render("page/editor", &()).unwrap())
    } else if let Some(access_token) = &query.token {
        if let Some(auth_token) = sell_token(&access_token) {
            HttpResponse::Ok()
                .cookie(
                    Cookie::build("auth", auth_token)
                        .http_only(true)
                        .max_age(Duration::days(30))
                        .finish(),
                )
                .body(template.render("page/editor", &()).unwrap())
        } else {
            HttpResponse::Unauthorized().body("No such token ID.")
        }
    } else {
        HttpResponse::Unauthorized().body("Could not get token.")
    }
}
