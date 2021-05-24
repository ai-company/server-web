use std::future::Future;

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse},
    HttpMessage, HttpRequest, HttpResponse,
};
use futures::future::Either;

use crate::tokens::is_invited;

pub fn authorized(
    req: ServiceRequest,
    srv: &mut impl Service<
        Request = ServiceRequest,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
) -> impl Future<Output = Result<ServiceResponse, actix_web::Error>> {
    let authorized = req
        .cookies()
        .map(|jar| {
            jar.iter()
                .find(|cookie| cookie.name() == "auth")
                .map(|auth| is_invited(auth.value()))
                .unwrap_or(false)
        })
        .unwrap_or(false);

    if authorized {
        Either::Left(srv.call(req))
    } else {
        Either::Right(async move {
            Ok(ServiceResponse::new(
                req.into_parts().0,
                HttpResponse::Unauthorized().finish(),
            ))
        })
    }
}

pub fn is_authorized(req: &HttpRequest) -> bool {
    req.cookies()
        .map(|jar| {
            jar.iter()
                .find(|cookie| cookie.name() == "auth")
                .map(|auth| is_invited(auth.value()))
                .unwrap_or(false)
        })
        .unwrap_or(false)
}
