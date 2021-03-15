mod asset;
mod editor;
mod feedback;
mod index;
mod not_found;
mod report;

use crate::{ai_client::AIClient, web::StaticAssetStore};
use chrono::prelude::*;

use hyper::{Body, Method, Request};
use std::sync::Arc;

pub struct SharedContext {
    pub root_path: String,
    pub asset_store: StaticAssetStore,
    pub model_danish: AIClient,
    pub model_english: AIClient,
}

pub type Response = Result<hyper::Response<Body>, hyper::http::Error>;

pub async fn route(req: Request<Body>, context: Arc<SharedContext>) -> Response {
    let method = req.method().as_str().to_owned();
    let path = req.uri().path().to_owned();
    let time = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let result = match (req.method(), req.uri().path()) {
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
        (&Method::POST, "/editor") => editor::post(req, &context).await,

        // editor feedback api endpoint
        // requires authorization
        (&Method::POST, "/feedback") => feedback::post(req, &context).await,

        // bad correction api endpoint
        // requires authentication
        (&Method::POST, "/report") => report::post(req, &context).await,

        // static assets
        (&Method::GET, path) if context.asset_store.has(path) => asset::get(req, &context).await,

        // 404 not found
        _ => not_found::get(req, &context).await,
    };

    match &result {
        Ok(response) => {
            let color = if response.status().as_u16() >= 400 {
                "\x1b[31m"
            } else if response.status().as_u16() == 304 {
                ""
            } else {
                "\x1b[32m"
            };

            println!(
                "\x1b[30;1m{} {}{:>7} {} {:.128}\x1b[0m",
                time,
                color,
                method,
                response.status().as_u16(),
                path
            );
        }
        Err(_) => println!("{} \x1b[41m{:7} --- {}\x1b[0m", time, method, path),
    }

    result
}
