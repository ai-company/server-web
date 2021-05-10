use crate::util;
use crate::{middleware, tokens::sell_token};

use crate::route::SharedContext;

use actix_web::{
    cookie::Cookie,
    web::{self, Query},
    HttpRequest, HttpResponse, Responder,
};
use serde::Deserialize;
use time::Duration;

#[derive(Deserialize)]
pub struct TokenQuery {
    token: Option<String>,
}

pub async fn get(
    req: HttpRequest,
    query: Query<TokenQuery>,
    context: web::Data<SharedContext>,
) -> impl Responder {
    if middleware::is_authorized(&req) {
        let template = util::get_template(&context, "editor").unwrap();
        HttpResponse::Ok().body(template)
    } else if let Some(access_token) = &query.token {
        if let Some(auth_token) = sell_token(&access_token) {
            let template = util::get_template(&context, "editor");

            HttpResponse::Ok()
                .cookie(
                    Cookie::build("auth", auth_token)
                        .http_only(true)
                        .max_age(Duration::days(30))
                        .finish(),
                )
                .body(template.unwrap())
        } else {
            HttpResponse::Unauthorized().body("No such token ID.")
        }
    } else {
        HttpResponse::Unauthorized().body("Could not get token.")
    }
}
