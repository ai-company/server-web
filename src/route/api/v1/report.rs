use actix_web::{client::Client, http::header, web::Data, HttpResponse, Responder};

use crate::{middleware::UserAuthorized, route::AiModels};

/// Correction error reporting route
///
/// RESTRICTED:
/// * UserAuthorized
/// * TODO: UserSubscriptionValid
///
/// FLOW:
/// * receive full text diff and index of the incorrect item
///     * if empty:
///         * return BadRequest
/// * try get db connection
///     * if no connection:
///         * return InternalServerError
/// * try store report in database
///     * if failed:
///         * return InternalServerError
/// * return OK
///
/// TYPES:
/// ```json
/// IN: {
///     "diff": [
///         {
///             "index": int,
///             "type": "none" | "add" | "remove" | "split" | "merge",
///             "origin": str | str[],
///             "change": str | this[],
///             "explain": str[]?
///         },
///         ...
///     ],
///     "id": int
/// ```
///
pub async fn post(
    body_string: String,
    _context: Data<AiModels>,
    _mw_authorized: UserAuthorized,
) -> impl Responder {
    let client = Client::new();

    client
        .post("http://127.0.0.1:6969/api/report")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send_body(body_string)
        .await;

    HttpResponse::Ok().body("Give me back my feet.")
}
