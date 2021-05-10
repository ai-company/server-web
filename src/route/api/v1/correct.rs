use actix_web::{web, HttpRequest, HttpResponse, Responder};

use crate::middleware;
use crate::route::SharedContext;

pub async fn post(
    req: HttpRequest,
    body_string: String,
    context: web::Data<SharedContext>,
) -> impl Responder {
    if !middleware::is_authorized(&req) {
        HttpResponse::Unauthorized().finish()
    } else {
        if body_string.trim().len() == 0 {
            HttpResponse::Ok().body("[]")
        } else {
            match context.model_danish.request(&body_string) {
                Ok(result) => HttpResponse::Ok().body(result),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}
