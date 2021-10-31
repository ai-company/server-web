use actix_web::{web, HttpResponse, Responder};

use crate::{middleware, route::AiModels};

/// Correction route
///
/// RESTRICTED:
/// * UserAuthorized
/// * TODO: UserSubscriptionValid
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
pub async fn post(
    body_string: String,
    context: web::Data<AiModels>,
    _: middleware::UserAuthorized,
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
