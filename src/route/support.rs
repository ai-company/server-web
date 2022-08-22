use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header;
use actix_web::{web, HttpRequest};
use actix_web::{web::Data, HttpResponse, Responder};

use handlebars::Handlebars;
use time::Duration;

use crate::database::{feedback, DBConnection, SqlPool};
use crate::emails::support::SupportEmail;
use crate::emails::Email;
use crate::helper::get_current_user_id;
use crate::middleware;
use crate::route::{EndpointProcessingError, EndpointProcessingResult};
use crate::util::FormData;

async fn support(
    req: HttpRequest,
    pool: &SqlPool,
    session: middleware::context::SessionContext,
    renderer: &Handlebars<'_>,
    form: &FormData,
) -> EndpointProcessingResult<()> {
    if let middleware::Session::Guest = session.user {
        return EndpointProcessingResult::Err(EndpointProcessingError::Unauthorized);
    }

    let user = session.user.unwrap_user();

    let result = match feedback::insert(
        pool.clone(),
        &user.email,
        form.get("comment").and_then(|f| f.string()).unwrap_or(""),
    ) {
        Ok(_) => Ok(()),
        Err(e) => EndpointProcessingResult::Err(EndpointProcessingError::Processing(e)),
    };

    SupportEmail::new(
        &user.email,
        form.get("comment").and_then(|f| f.string()).unwrap_or(""),
    )
    .render(&renderer)
    .send("support@orto.ai");

    result
}

// Wrapper

pub async fn post(
    req: HttpRequest,
    _template: Data<Handlebars<'_>>,
    pool: web::Data<SqlPool>,
    session: middleware::context::SessionContext,
    form: FormData,
    _: middleware::UserAuthorized,
) -> impl Responder {
    use EndpointProcessingError::*;

    match support(req, pool.as_ref(), session, &_template, &form).await {
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
