mod api;
mod editor;
mod index;
mod mailtest;
mod not_found;
pub mod payment;
pub mod user;

use std::{
    fmt::{self, Display, Formatter},
};

use actix_files::Files;
use actix_web::{dev::HttpServiceFactory, web};

use crate::{ai_client::AIClient, middleware::logger};

#[derive(Clone, Debug)]
pub struct AiModels {
    pub model_danish: AIClient,
    pub model_english: AIClient,
}

#[derive(Clone, Debug)]
pub struct RootPath(pub String);

impl RootPath {
    pub fn new<T: Into<String>>(path: T) -> Self {
        RootPath(path.into())
    }
}

impl Display for RootPath {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(fmt)
    }
}

impl Into<RootPath> for String {
    fn into(self) -> RootPath {
        RootPath::new(self)
    }
}

impl Into<String> for RootPath {
    fn into(self) -> String {
        self.0
    }
}

#[cfg(debug_assertions)]
pub const BASE_URL: &str = "http://local.host:8000";
#[cfg(not(debug_assertions))]
pub const BASE_URL: &str = "https://orto.ai";

pub fn url(path: &str) -> String {
    format!("{}/{}", BASE_URL, path.trim_matches('/'))
}

pub fn router() -> impl HttpServiceFactory {
    // static assets
    let assets = Files::new("/static", "./public/static").prefer_utf8(true);

    // home landing
    // if not authorized: serve form
    // else: serve button to editor
    let landing = web::resource("/").route(web::get().to(index::get));

    // editor
    // if authorized: serve editor
    // else if not authorized but has valid token param: consume token and authorize
    // else: redirect home with sad message
    let editor = web::resource("/editor/").route(web::get().to(editor::get));

    let user = web::scope("/user/")
        .service(
            web::scope("/signup/")
                .route("", web::get().to(user::signup::index::get))
                .route("", web::post().to(user::signup::index::post)),
        )
        .service(
            web::resource("/signin/")
                .route(web::get().to(user::signin::get))
                .route(web::post().to(user::signin::post)),
        )
        .service(web::resource("/signout/").route(web::get().to(user::signout::get)))
        .service(web::resource("/delete/").route(web::get().to(user::delete::get)))
        .service(
            web::resource("/account/")
                .route(web::get().to(user::account::get))
                .route(web::delete().to(user::account::delete)),
        );

    let stripe = web::scope("/payment/")
        .route("", web::get().to(payment::index::get))
        .route("/success/", web::get().to(payment::success::get))
        .route("/cancel/", web::get().to(payment::cancel::get))
        .service(web::resource("/status/{user_id}/").route(web::get().to(payment::status::get)))
        .service(
            web::scope("/stripe/")
                .service(
                    web::resource("/webhook/")
                        .route(web::post().to(payment::stripe::webhook::post)),
                )
                // .service(
                //     web::resource("/checkout/")
                //         .route(web::post().to(payment::stripe::checkout::post)),
                // )
                .service(
                    web::resource("/portal/").route(web::post().to(payment::stripe::portal::post)),
                ),
        );

    // api endpoints
    let api = web::scope("/api/")
        .service(
            // requires authorization
            web::scope("/v1/")
                .service(
                    web::resource("/correct/")
                        // get orto corrections
                        .route(web::post().to(api::v1::correct::post)),
                )
                .service(
                    web::resource("/report/")
                        // bad correction reports
                        .route(web::post().to(api::v1::report::post)),
                ),
        )
        .service(
            // demo api, ratelimited
            web::resource("/demo/correct/").route(web::post().to(api::demo::correct::post)),
        );

    let routes = web::scope("")
        .wrap_fn(logger)
        .service(assets)
        .service(landing)
        .service(editor)
        .service(user)
        .service(stripe)
        .service(api)
        .default_service(web::to(not_found::get));

    #[cfg(debug_assertions)]
    let routes = {
        let mail_test = web::resource("/mailtest/").route(web::get().to(mailtest::get));

        routes.service(mail_test)
    };

    routes
}

#[derive(Debug)]
pub enum EndpointProcessingError {
    Unauthorized,
    RequestUnparsable,
    RequestNonsensical,
    Processing(Box<dyn std::error::Error>),
}

impl From<Box<dyn std::error::Error>> for EndpointProcessingError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        EndpointProcessingError::Processing(e)
    }
}

impl From<actix_web::Error> for EndpointProcessingError {
    fn from(e: actix_web::Error) -> Self {
        EndpointProcessingError::Processing(Box::new(e))
    }
}
impl From<actix_web::http::Error> for EndpointProcessingError {
    fn from(e: actix_web::http::Error) -> Self {
        EndpointProcessingError::Processing(Box::new(e))
    }
}
impl From<actix_web::cookie::ParseError> for EndpointProcessingError {
    fn from(e: actix_web::cookie::ParseError) -> Self {
        EndpointProcessingError::Processing(Box::new(e))
    }
}

impl From<serde_qs::Error> for EndpointProcessingError {
    fn from(e: serde_qs::Error) -> Self {
        EndpointProcessingError::Processing(Box::new(e))
    }
}

impl From<serde_json::Error> for EndpointProcessingError {
    fn from(e: serde_json::Error) -> Self {
        EndpointProcessingError::Processing(Box::new(e))
    }
}

impl std::fmt::Display for EndpointProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use EndpointProcessingError::*;
        match self {
            Unauthorized => write!(f, "User was not authorized"),
            RequestUnparsable => write!(f, "Request did not match expected structure"),
            RequestNonsensical => write!(f, "Did not expect request"),
            Processing(e) => write!(f, "Encountered error during processing: {}", e),
        }
    }
}

impl std::error::Error for EndpointProcessingError {}

pub type EndpointProcessingResult<T> = Result<T, EndpointProcessingError>;
