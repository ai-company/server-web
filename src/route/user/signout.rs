use actix_web::http::header;
use actix_web::{web, HttpRequest};
use actix_web::{web::Data, HttpResponse, Responder};

use handlebars::Handlebars;

use crate::database::{sessions, SqlPool};
use crate::helper::get_current_user_id;
use crate::middleware;
use crate::route::{EndpointProcessingError, EndpointProcessingResult};

async fn signout(req: HttpRequest, pool: &SqlPool) -> EndpointProcessingResult<()> {
    let user_id = get_current_user_id(&req, pool)?;
    sessions::destroy(pool, user_id)?;
    Ok(())
}

// Wrapper

pub async fn get(
    req: HttpRequest,
    _template: Data<Handlebars<'_>>,
    pool: web::Data<SqlPool>,
    _: middleware::UserAuthorized,
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
