use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header;
use actix_web::{web, HttpRequest};
use actix_web::{web::Data, HttpResponse, Responder};

use handlebars::Handlebars;
use time::Duration;

use crate::database::{correction_reports, DBConnection, SqlPool};
use crate::emails::support::SupportEmail;
use crate::emails::Email;
use crate::helper::get_current_user_id;
use crate::middleware;
use crate::route::{EndpointProcessingError, EndpointProcessingResult};
use crate::util::FormData;

async fn report(
    req: HttpRequest,
    pool: &SqlPool,
    session: middleware::Session,
    data: &str,
) -> EndpointProcessingResult<()> {
    if let middleware::Session::Guest = session {
        return EndpointProcessingResult::Err(EndpointProcessingError::Unauthorized);
    }

    let user = session.unwrap_user();

    let result = match correction_reports::insert(pool.clone(), &user.email, data) {
        Ok(_) => Ok(()),
        Err(e) => EndpointProcessingResult::Err(EndpointProcessingError::Processing(e)),
    };

    result
}

// Wrapper

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
    req: HttpRequest,
    pool: web::Data<SqlPool>,
    session: middleware::Session,
    data: String,
    _: middleware::UserAuthorized,
) -> impl Responder {
    use EndpointProcessingError::*;

    match report(req, pool.as_ref(), session, &data).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(Unauthorized) => HttpResponse::Found().header(header::LOCATION, "/").finish(),
        Err(RequestUnparsable) | Err(RequestNonsensical) => HttpResponse::BadRequest().finish(),
        Err(e) => {
            println!(
                "Failed to submit support/feedback comment with error: {}",
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
