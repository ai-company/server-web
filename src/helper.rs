use actix_web::{HttpMessage, HttpRequest};

use crate::{
    database::{
        admin::{self, Admin, AdminID},
        admin_sessions, sessions,
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

/// Get current `session` cookie if one exists
pub fn get_session_cookie(req: &HttpRequest) -> Option<String> {
    req.cookies()
        .map(|jar| {
            jar.iter()
                .find(|cookie| cookie.name() == "session")
                .map(|auth| auth.value().into())
        })
        .unwrap_or(None)
}

/// Find current `UserID` associated with the current `session` cookie
pub fn get_current_user_id(
    req: &HttpRequest,
    pool: &SqlPool,
) -> Result<UserID, EndpointProcessingError> {
    let auth = match get_session_cookie(req) {
        Some(v) => v,
        None => return Err(EndpointProcessingError::Unauthorized),
    };

    let id = sessions::get_owner(pool, auth)?;

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
        Ok(None) => Err(EndpointProcessingError::RequestNonsensical),
        Err(e) => Err(e.into()),
    }
}

/// Get current `admin` cookie if one exists
pub fn get_admin_session_cookie(req: &HttpRequest) -> Option<String> {
    req.cookies()
        .map(|jar| {
            jar.iter()
                .find(|cookie| cookie.name() == "admin")
                .map(|auth| auth.value().into())
        })
        .unwrap_or(None)
}

/// Find current `UserID` associated with the current `session` cookie
pub fn get_current_admin_id(
    req: &HttpRequest,
    pool: &SqlPool,
) -> Result<AdminID, EndpointProcessingError> {
    let auth = match get_admin_session_cookie(req) {
        Some(v) => v,
        None => return Err(EndpointProcessingError::Unauthorized),
    };

    let id = admin_sessions::get_owner(pool, auth)?;

    if let Some(id) = id {
        Ok(id)
    } else {
        Err(EndpointProcessingError::Unauthorized)
    }
}

/// Find `Admin` associated with the current `AdminID`
pub fn get_current_admin(
    req: &HttpRequest,
    pool: &SqlPool,
) -> Result<Admin, EndpointProcessingError> {
    let id = get_current_admin_id(req, pool)?;

    match admin::get_with_id(id, pool) {
        Ok(Some(v)) => Ok(v),
        Ok(None) => Err(EndpointProcessingError::RequestNonsensical),
        Err(e) => Err(e.into()),
    }
}
