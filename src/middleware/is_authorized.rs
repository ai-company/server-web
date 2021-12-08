use crate::{database::SqlPool, helper::get_current_user_id};
use actix_web::{dev, error::InternalError, web, FromRequest, HttpRequest, HttpResponse};
use futures::future::{err, ok, Ready};
use handlebars::Handlebars;

/// Route Authorization middleware
///
/// The route that contains this extractor will only execute if user is logged on
///
/// Usage:
/// ```
/// fn route_handler(_: HttpRequest, _mw_auth: UserAuthorized) -> impl Responder { ... }
/// ```
///
/// Flow:
/// * try get db connection
///     * if no connection:
///         * return InternalServerError with direct html body in case there are bigger issues
/// * try get current user session
///     * if no session:
///         * return Unauthorized with 401 template
/// * continue
pub struct UserAuthorized;

impl FromRequest for UserAuthorized {
    type Error = InternalError<&'static str>;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(_req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let pool = _req.app_data::<web::Data<SqlPool>>();
        let template = _req.app_data::<web::Data<Handlebars<'_>>>().unwrap();

        if let Some(pool) = pool {
            match get_current_user_id(_req, pool) {
                Ok(_) => ok(UserAuthorized),
                Err(_) => err(InternalError::from_response(
                    "unauthorized",
                    HttpResponse::Unauthorized().body(template.render("page/401", &()).unwrap()),
                )),
            }
        } else {
            println!("unable to get db");
            err(InternalError::from_response(
                "internal error",
                HttpResponse::InternalServerError()
                    .body("<h1>Vi døjer med nogle midlertidige server-problemer.</h1>"),
            ))
        }
    }
}
