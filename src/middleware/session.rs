use crate::database::billing_info::{self, BillingInfo};
use crate::database::user::User;
use crate::helper::get_current_user;

use crate::{database::SqlPool};
use actix_web::{
    dev, error::InternalError, web, FromRequest, HttpRequest, HttpResponse,
};
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
/// * try get current user session
///     * if no session:
///         * return Guest default
/// * try fetch additional user data
///     * if failed:
///         * return InternalServerError with direct html body in case there are bigger issues
/// * continue
#[derive(Debug, Serialize, Deserialize)]
pub enum Session {
    Guest,
    User(SessionData),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub user: User,
    pub billing: BillingInfo,
}

impl Session {
    pub fn unwrap_user(&self) -> &User {
        match self {
            Self::User(data) => &data.user,
            _ => panic!("user session not active!"),
        }
    }

    pub fn unwrap_billing(&self) -> &BillingInfo {
        match self {
            Self::User(data) => &data.billing,
            _ => panic!("user session not active!"),
        }
    }
}

impl FromRequest for Session {
    type Error = InternalError<&'static str>;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(_req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let pool = _req.app_data::<web::Data<SqlPool>>();
        let _template = _req.app_data::<web::Data<Handlebars<'_>>>().unwrap();

        if let Some(pool) = pool {
            match get_current_user(_req, pool) {
                Ok(user) => {
                    let billing = billing_info::get_with_id(user.id, pool);
                    ok(Session::User(SessionData {
                        user,
                        billing: billing.unwrap().unwrap(),
                    }))
                }
                Err(_) => ok(Session::Guest),
            }
        } else {
            println!("unable to get db");
            err(InternalError::from_response(
                "internal error",
                HttpResponse::InternalServerError()
                    .body("<h1>Internal Server Error, please stand by</h1>"),
            ))
        }
    }
}
