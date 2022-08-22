use crate::database::admin::Admin;
use crate::helper::get_current_admin;

use crate::database::SqlPool;
use actix_web::{dev, error::InternalError, web, FromRequest, HttpRequest, HttpResponse};
use futures::future::{err, ok, Ready};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

/// Session fetching middleware
///
/// The route that contains this extractor will fetch an user's data, if authenticated, otherwise provide guest default
///
/// Usage:
/// ```
/// fn route_handler(_: HttpRequest, session: Session) -> impl Responder { ... }
/// ```
///
/// Flow:
/// * try get db connection
///     * if no connection:
///         * return InternalServerError with direct html body in case there are bigger issues
/// * try get current admin session
///     * if no session:
///         * return Guest default
/// * try fetch additional admin data
///     * if failed:
///         * return InternalServerError with direct html body in case there are bigger issues
/// * continue
#[derive(Debug, Serialize, Deserialize)]
pub enum AdminSession {
    Guest,
    Admin(AdminSessionData),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminSessionData {
    pub user: Admin,
}

impl AdminSession {
    pub fn unwrap_user(&self) -> &Admin {
        match self {
            Self::Admin(data) => &data.user,
            _ => panic!("admin session not active!"),
        }
    }
}

impl FromRequest for AdminSession {
    type Error = InternalError<&'static str>;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(_req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let pool = _req.app_data::<web::Data<SqlPool>>();
        let _template = _req.app_data::<web::Data<Handlebars<'_>>>().unwrap();

        if let Some(pool) = pool {
            match get_current_admin(_req, pool) {
                Ok(user) => ok(AdminSession::Admin(AdminSessionData { user })),
                Err(_) => ok(AdminSession::Guest),
            }
        } else {
            println!("unable to get db");
            err(InternalError::from_response(
                "internal error",
                HttpResponse::InternalServerError().body(
                    "<h1>Vi døjer med nogle midlertidige server-problemer, prøv igen om lidt!</h1>",
                ),
            ))
        }
    }
}
