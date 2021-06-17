mod api;
mod editor;
mod index;
mod not_found;
pub mod payment;
mod user;

use std::fmt::{self, Display, Formatter};

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

pub fn router() -> impl HttpServiceFactory {
    web::scope("")
        .wrap_fn(logger)
        .service(
            // static assets
            Files::new("/static", "./public/static").prefer_utf8(true),
        )
        .service(
            web::resource("/")
                // home landing
                // if not authorized: serve form
                // else: serve button to editor
                .route(web::get().to(index::get)),
        )
        .service(
            web::resource("/editor/")
                // editor
                // if authorized: serve editor
                // else if not authorized but has valid token param: consume token and authorize
                // else: redirect home with sad message
                .route(web::get().to(editor::get)),
        )
        .service(
            web::scope("/user/")
                .service(
                    web::resource("/signup/")
                        .route(web::get().to(user::signup::get))
                        .route(web::post().to(user::signup::post)),
                )
                .service(
                    web::resource("/signin/")
                        .route(web::get().to(user::signin::get))
                        .route(web::post().to(user::signin::post)),
                )
                .service(
                    web::resource("/account/")
                        .route(web::get().to(user::account::get))
                        .route(web::delete().to(user::account::delete)),
                ),
        )
        .service(
            web::scope("/payment/")
                .service(
                    web::resource("/status/{user_id}/").route(web::get().to(payment::status::get)),
                )
                .service(
                    web::scope("/stripe/")
                        .service(
                            web::resource("/webhook/")
                                .route(web::post().to(payment::stripe::webhook::post)),
                        )
                        .service(
                            web::resource("/checkout/")
                                .route(web::post().to(payment::stripe::checkout::post)),
                        )
                        .service(
                            web::resource("/portal/")
                                .route(web::post().to(payment::stripe::portal::post)),
                        ),
                ),
        )
        .service(
            // api endpoints
            web::scope("/api/")
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
                ),
        )
        .default_service(web::to(not_found::get))
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
