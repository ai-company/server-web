use crate::database::billing_info::{self, BillingInfo};
use crate::database::user::User;
use crate::helper::get_current_user;

use crate::database::SqlPool;
use crate::middleware::{AdminSession, Session};
use actix_web::{dev, error::InternalError, web, FromRequest, HttpRequest, HttpResponse};
use futures::future::{err, ok, Ready};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionContext {
    pub user: Session,
    pub admin: AdminSession,
}

impl FromRequest for SessionContext {
    type Error = InternalError<&'static str>;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(_req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let user = Session::from_request(_req, _payload).into_inner();
        let admin = AdminSession::from_request(_req, _payload).into_inner();

        match (user, admin) {
            (Err(e), _) | (_, Err(e)) => err(e),
            (Ok(user), Ok(admin)) => ok(Self { user, admin }),
        }
    }
}
