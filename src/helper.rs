use actix_web::{HttpMessage, HttpRequest};

use crate::{
    database::{
        token,
        user::{self, User, UserID},
        SqlPool,
    },
    route::EndpointProcessingError,
};

crate::basic_error!(
    UnexpectedTimeTravel,
    "UnexpectedTimeTravel: This should never occour after the Unix Epoch, something is very wrong"
);

pub fn get_millis_since_epoch() -> Result<i64, UnexpectedTimeTravel> {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(v) => Ok(v.as_millis() as i64),
        Err(_) => Err(UnexpectedTimeTravel),
    }
}

/// Get current `auth` cookie if one exists
pub fn get_session_cookie(req: &HttpRequest) -> Option<String> {
    req.cookies()
        .map(|jar| {
            jar.iter()
                .find(|cookie| cookie.name() == "auth")
                .map(|auth| auth.value().into())
        })
        .unwrap_or(None)
}

/// Find current `UserID` associated with the current `auth` cookie
pub fn get_current_user_id(
    req: &HttpRequest,
    pool: &SqlPool,
) -> Result<UserID, EndpointProcessingError> {
    let auth = match get_session_cookie(req) {
        Some(v) => v,
        None => return Err(EndpointProcessingError::Unauthorized),
    };

    let id = token::get_owner(pool, auth)?;

    if let Some(id) = id {
        Ok(id)
    } else {
        Err(EndpointProcessingError::Unauthorized)
    }
}

/// Find `User` associated with the current `UserID`
pub fn get_current_user(
    req: &HttpRequest,
    pool: &SqlPool,
) -> Result<User, EndpointProcessingError> {
    let id = get_current_user_id(req, pool)?;

    match user::get_with_id(id, pool) {
        Ok(Some(v)) => Ok(v),
        Ok(None) => return Err(EndpointProcessingError::RequestNonsensical),
        Err(e) => return Err(e.into()),
    }
}
