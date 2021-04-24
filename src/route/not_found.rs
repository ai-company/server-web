use crate::web;

use crate::route::{Response, SharedContext};

use hyper::{Body, Request, StatusCode};

pub async fn get(_req: Request<Body>, context: &SharedContext) -> Response {
    let template = web::get_template(&context, "404").unwrap();

    hyper::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(template))
}
