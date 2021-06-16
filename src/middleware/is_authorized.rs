use std::future::Future;

use actix_web::{
    dev,
    dev::{Service, ServiceRequest, ServiceResponse},
    error::InternalError,
    web, FromRequest, HttpMessage, HttpRequest, HttpResponse,
};
use futures::future::Either;

use crate::tokens::is_invited;
use futures::future::{err, ok, Ready};
use handlebars::Handlebars;

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
use crate::{database::SqlPool, helper::get_current_user_id};

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

pub struct UserAuthorized;

impl FromRequest for UserAuthorized {
    type Error = InternalError<&'static str>;
    type Future = Ready<Result<UserAuthorized, Self::Error>>;
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
                    .body("<h1>Internal Server Error, please stand by</h1>"),
            ))
        }
    }
}
