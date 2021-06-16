use crate::tokens::is_invited;
use crate::{database::SqlPool, helper::get_current_user_id};
use actix_web::{
    dev, error::InternalError, web, FromRequest, HttpMessage, HttpRequest, HttpResponse,
};
use futures::future::{err, ok, Ready};
use handlebars::Handlebars;

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
