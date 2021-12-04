use actix_web::{web, HttpResponse, Responder};

use crate::route::AiModels;

/// Demo Correction route
///
/// RESTRICTED:
/// * TODO: DemoLimitRate
/// * TODO: DemoLimitInput
///
/// FLOW:
/// * receive plain string sentence body
///     * if empty:
///         * return OK []
/// * call model server
///     * if errored:
///         * return InternalServerError
/// * return OK Json response
///
/// TYPES:
/// ```json
/// IN: String
/// OUT: [
///     {
///         "index": int,
///         "type": "none" | "add" | "remove" | "split" | "merge",
///         "origin": str | str[],
///         "change": str | this[],
///         "explain": str[]?
///     },
///     ...
/// ]
/// ```
///
/// TODO: implement ratelimiting (5 requests per minute)
///
/// TODO: implement character limit (512ch)
pub async fn post(body_string: String, context: web::Data<AiModels>) -> impl Responder {
    if body_string.trim().is_empty() {
        HttpResponse::Ok().body("[]")
    } else {
        match context.model_danish.request(&body_string) {
            Ok(result) => HttpResponse::Ok().body(result),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    }
}
