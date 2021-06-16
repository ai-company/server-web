use actix_web::{web, HttpResponse, Responder};

use crate::{middleware::UserAuthorized, route::SharedContext};

pub async fn post(
    body_string: String,
    context: web::Data<SharedContext>,
    _mw_authorized: UserAuthorized,
) -> impl Responder {
    if body_string.trim().len() == 0 {
        HttpResponse::Ok().body("[]")
    } else {
        match context.model_danish.request(&body_string) {
            Ok(result) => HttpResponse::Ok().body(result),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    }
}
