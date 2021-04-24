use crate::route::{Response, SharedContext};

use hyper::{header, Body, Request, StatusCode};

pub async fn get(req: Request<Body>, context: &SharedContext) -> Response {
    let path = req.uri().path();

    // check if browser has a cached version that matches ours
    if let Some(mtime) = req.headers().get(header::IF_MODIFIED_SINCE) {
        if mtime.to_str().unwrap() == context.asset_store.mtime(path) {
            return hyper::Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .body(Body::from(""));
        }
    }

    let asset = context.asset_store.get(path).await.unwrap();

    hyper::Response::builder()
        .header(header::CONTENT_TYPE, asset.mime)
        .header(header::LAST_MODIFIED, asset.mtime)
        .body(Body::from(asset.data))
}
