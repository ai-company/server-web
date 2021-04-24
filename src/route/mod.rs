mod api;
mod asset;
mod editor;
mod feedback;
mod index;
mod not_found;
mod report;

use crate::{ai_client::AIClient, web::StaticAssetStore};
use chrono::prelude::*;

use hyper::{Body, Method, Request};
use std::convert::TryFrom;
use std::sync::Arc;

pub struct SharedContext {
    pub root_path: String,
    pub asset_store: StaticAssetStore,
    pub model_danish: AIClient,
    pub model_english: AIClient,
}

pub type Response = Result<hyper::Response<Body>, hyper::http::Error>;

pub async fn route(mut req: Request<Body>, context: Arc<SharedContext>) -> Response {
    let start_time = Utc::now().time();
    let date = Utc::now().format("%d/%m/%y %T").to_string();

    // extract and normalise path then reconstitute uri
    {
        let path_and_query = req.uri().path_and_query().unwrap();
        let mut path = uriparse::Path::try_from(path_and_query.path()).unwrap();
        path.normalize(true);

        let path = path.to_string().replace("//", "/");
        if let Some(query) = path_and_query.query() {
            *req.uri_mut() = format!("{}?{}", path.to_string(), query).parse().unwrap();
        } else {
            *req.uri_mut() = format!("{}", path.to_string()).parse().unwrap();
        }
    }

    let method = req.method().as_str().to_owned();
    let path = req.uri().path().to_owned();

    let result = match (req.method(), req.uri().path()) {
        // static assets
        (&Method::GET, path) if context.asset_store.has(path) => asset::get(req, &context).await,

        // home landing
        // if not authorized: serve form
        // else: serve button to editor
        (&Method::GET, "/") => index::get(req, &context).await,

        // closed-alpha email authorization
        // if authorized: redirect to editor
        // else if not authorized but email in alpha list: send email
        // else: redirect home with sad message
        (&Method::POST, "/") => index::post(req, &context).await,

        // editor
        // if authorized: serve editor
        // else if not authorized but has valid token param: consume token and authorize
        // else: redirect home with sad message
        (&Method::GET, "/editor") => editor::get(req, &context).await,

        // correction api endpoint
        // requires authorization
        (&Method::POST, "/api/v1/correct") => api::v1::correct::post(req, &context).await,

        // editor feedback api endpoint
        // requires authorization
        (&Method::POST, "/feedback") => feedback::post(req, &context).await,

        // bad correction api endpoint
        // requires authentication
        (&Method::POST, "/report") => report::post(req, &context).await,

        // 404 not found
        _ => not_found::get(req, &context).await,
    };

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
