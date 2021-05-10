use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

use crate::middleware::is_authorized;
use crate::tokens;
use crate::util;

use crate::route::SharedContext;

pub async fn get(req: HttpRequest, context: web::Data<SharedContext>) -> impl Responder {
    let template = util::get_template(
        &context,
        if is_authorized(&req) {
            "index-authorized"
        } else {
            "index"
        },
    )
    .unwrap();

    HttpResponse::Ok().body(template)
}

#[derive(Deserialize)]
pub struct AlphaEmail {
    email: String,
}

pub async fn post(req: web::Form<AlphaEmail>) -> impl Responder {
    match tokens::try_send_token_to(&req.email) {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::Unauthorized(),
    }
}
