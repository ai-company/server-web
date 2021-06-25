use actix_web::web;
use actix_web::{web::Data, HttpRequest, HttpResponse, Responder};

use crate::database::{user, SqlPool};
use crate::helper::get_current_user;

use crate::middleware::{Session, UserAuthorized};
use crate::route::{EndpointProcessingError, EndpointProcessingResult};

use handlebars::Handlebars;

// Business Logic

crate::basic_error!(TestError, "test");

async fn account_delete(req: HttpRequest, pool: &SqlPool) -> EndpointProcessingResult<()> {
    let target = get_current_user(&req, pool)?;

    // Then we delete them from the database(This will also trigger deletion from payment profiles)
    // user::delete(target, pool).await.map_err(|e| e.into())
    match user::delete(target, pool).await {
        Ok(_) => Ok(()),
        Err(_) => Err(EndpointProcessingError::Processing(Box::new(TestError))),
    }
}

// Wrappers

pub async fn get(
    template: Data<Handlebars<'_>>,
    _mw_authorized: UserAuthorized,
    session: Session,
) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/user/account", &session).unwrap())
}

pub async fn delete(
    req: HttpRequest,
    pool: web::Data<SqlPool>,
    _mw_authorized: UserAuthorized,
) -> impl Responder {
    use EndpointProcessingError::*;
    match account_delete(req, pool.as_ref()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(Unauthorized) => HttpResponse::Unauthorized().finish(),
        Err(RequestUnparsable) | Err(RequestNonsensical) => HttpResponse::BadRequest().finish(),
        Err(e) => {
            println!("Failed to create user with error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
