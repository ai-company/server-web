mod api;
mod editor;
mod index;
mod not_found;

use actix_files::Files;
use actix_web::{dev::HttpServiceFactory, web};

use crate::{
    ai_client::AIClient,
    middleware::{authorized, logger},
};

#[derive(Clone, Debug)]
pub struct SharedContext {
    pub root_path: String,
    pub model_danish: AIClient,
    pub model_english: AIClient,
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
                .route(web::get().to(index::get))
                // closed-alpha email authorization
                // if authorized: redirect to editor
                // else if not authorized but email in alpha list: send email
                // else: redirect home with sad message
                .route(web::post().to(index::post)),
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
            // api endpoints
            // requires authorization, except demo
            web::scope("/api/")
                .service(
                    web::scope("/v1/")
                        .wrap_fn(authorized)
                        .service(
                            web::resource("/correct/")
                                // get orto corrections
                                .route(web::post().to(api::v1::correct::post)),
                        )
                        .service(
                            web::resource("/feedback/")
                                // editor feedback
                                .route(web::post().to(api::v1::feedback::post)),
                        )
                        .service(
                            web::resource("/report/")
                                // bad correction reports
                                .route(web::post().to(api::v1::report::post)),
                        ),
                ),
        )
        .default_service(web::to(not_found::get))
}
