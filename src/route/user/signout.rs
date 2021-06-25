use actix_web::cookie::Cookie;
use actix_web::http::header;
use actix_web::{web, HttpRequest};
use actix_web::{web::Data, HttpResponse, Responder};

use handlebars::Handlebars;
use time::Duration;

use serde::Deserialize;

use crate::database::{token, user, SqlPool};
use crate::helper::get_current_user_id;
use crate::middleware::UserAuthorized;
use crate::route::{EndpointProcessingError, EndpointProcessingResult};
use token::Token;
use user::User;

async fn signout(req: HttpRequest, pool: &SqlPool) -> EndpointProcessingResult<()> {
    let user_id = get_current_user_id(&req, &pool)?;
    token::destroy(pool, user_id)?;
    Ok(())
}

// Wrapper

pub async fn get(
    req: HttpRequest,
    template: Data<Handlebars<'_>>,
    pool: web::Data<SqlPool>,
    _mw_authorized: UserAuthorized,
) -> impl Responder {
    use EndpointProcessingError::*;

    match signout(req, pool.as_ref()).await {
        Ok(()) => HttpResponse::Found()
            .set_header(header::LOCATION, "/")
            .finish(),
        Err(Unauthorized) => HttpResponse::Found().header(header::LOCATION, "/").finish(),
        Err(RequestUnparsable) | Err(RequestNonsensical) => HttpResponse::BadRequest().finish(),
        Err(e) => {
            println!("Failed to sign out with error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
