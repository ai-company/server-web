use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header;
use actix_web::{web, HttpRequest};
use actix_web::{web::Data, HttpResponse, Responder};
use handlebars::Handlebars;
use time::Duration;

use crate::database::{user, SqlPool, SyncTransaction};
use crate::emails::{account::deletion::AccountDeletionEmail, Email, Envelope};
use crate::helper::get_current_user;
use crate::middleware;
use crate::route::{EndpointProcessingError, EndpointProcessingResult};

async fn delete(
    req: HttpRequest,
    pool: &SqlPool,
    renderer: &Handlebars<'_>,
) -> EndpointProcessingResult<()> {
    let user = get_current_user(&req, pool)?;
    let trans = SyncTransaction::new(pool)?;
    let email = user.email.clone();

    user::delete(user, &trans).await?;

    trans
        .commit()
        .map_err(|e| EndpointProcessingError::Processing(Box::new(e)))?;

    AccountDeletionEmail::new()
        .render(renderer)
        .send(&email)
        .map_err(|e| EndpointProcessingError::Processing(Box::new(e)))?;

    Ok(())
}

// Wrapper

pub async fn post(
    req: HttpRequest,
    template: Data<Handlebars<'_>>,
    pool: web::Data<SqlPool>,
    _: middleware::UserAuthorized,
) -> impl Responder {
    use EndpointProcessingError::*;

    match delete(req, pool.as_ref(), &template).await {
        Ok(()) => HttpResponse::Ok()
            .cookie(
                Cookie::build("session", "") // clear session cookie on logout
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .max_age(Duration::days(0))
                    .finish(),
            )
            .body(template.render("page/user/thankyou", &()).unwrap()),
        Err(Unauthorized) => HttpResponse::Found().header(header::LOCATION, "/").finish(),
        Err(RequestUnparsable) | Err(RequestNonsensical) => HttpResponse::BadRequest().finish(),
        Err(e) => {
            println!("Failed to delete user with error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
