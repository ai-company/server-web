mod api;
mod editor;
mod index;
mod not_found;

use std::future::Future;

use actix_files::Files;
use actix_web::{
    dev::{HttpServiceFactory, Service, ServiceRequest, ServiceResponse},
    web,
};
use chrono::prelude::*;

use crate::ai_client::AIClient;

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

fn logger(
    req: ServiceRequest,
    srv: &mut impl Service<
        Request = ServiceRequest,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
) -> impl Future<Output = Result<ServiceResponse, actix_web::Error>> {
    let start_time = Utc::now().time();
    let date = Utc::now().format("%d/%m/%y %T").to_string();
    let method = req.method().as_str().to_owned();
    let path = req.uri().path().to_owned();

    let fut = srv.call(req);

    async move {
        let result = fut.await;

        match &result {
            Ok(response) => {
                let end_time = Utc::now().time();
                let time = end_time - start_time;

                let color = if response.status().as_u16() >= 400 {
                    "\x1b[31m"
                } else if response.status().as_u16() == 304 {
                    ""
                } else {
                    "\x1b[32m"
                };

                println!(
                    "{}{} {:>6}µs {:>7} {} {:.128}\x1b[0m",
                    color,
                    date,
                    time.num_microseconds().unwrap(),
                    method,
                    response.status().as_u16(),
                    path
                );
            }
            Err(_) => println!("{} \x1b[41m{:7} --- {}\x1b[0m", date, method, path),
        }

        result
    }
}
