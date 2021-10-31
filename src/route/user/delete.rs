use actix_web::cookie::Cookie;
use actix_web::http::header;
use actix_web::{web, HttpRequest};
use actix_web::{web::Data, HttpResponse, Responder};

use handlebars::Handlebars;
use time::Duration;

use serde::Deserialize;

use crate::database::{sessions, user, SqlPool, SyncTransaction};
use crate::helper::{get_current_user, get_current_user_id};
use crate::middleware;
use crate::route::{EndpointProcessingError, EndpointProcessingResult};
use sessions::SessionToken;
use user::User;

async fn delete(req: HttpRequest, pool: &SqlPool) -> EndpointProcessingResult<()> {
    let user = get_current_user(&req, &pool)?;
    let trans = SyncTransaction::new(pool)?;

    user::delete(user, &trans).await?;

    trans
        .commit()
        .map_err(|e| EndpointProcessingError::Processing(Box::new(e)))?;

    Ok(())
}

// Wrapper

pub async fn get(
    req: HttpRequest,
    template: Data<Handlebars<'_>>,
    pool: web::Data<SqlPool>,
    _: middleware::UserAuthorized,
) -> impl Responder {
    use EndpointProcessingError::*;

    match delete(req, pool.as_ref()).await {
        Ok(()) => HttpResponse::Ok().body(template.render("page/user/thankyou", &()).unwrap()),
        Err(Unauthorized) => HttpResponse::Found().header(header::LOCATION, "/").finish(),
        Err(RequestUnparsable) | Err(RequestNonsensical) => HttpResponse::BadRequest().finish(),
        Err(e) => {
            println!("Failed to delete user with error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
