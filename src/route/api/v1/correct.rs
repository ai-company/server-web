use crate::middleware;

use crate::route::{Response, SharedContext};

use hyper::{body::HttpBody, header, Body, Request, StatusCode};

pub async fn post(req: Request<Body>, context: &SharedContext) -> Response {
    if !middleware::is_authorized(&req) {
        hyper::Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(""))
    } else {
        let body_vector: Vec<u8> = match req.into_body().data().await {
            Some(result) => result.unwrap().to_vec(),
            None => vec![],
        };
        let body_string = String::from_utf8(body_vector).unwrap();

        if body_string.trim().len() == 0 {
            return hyper::Response::builder()
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from(String::new()));
        }

        match context.model_danish.request(&body_string) {
            Ok(result) => hyper::Response::builder()
                .header(header::CONTENT_TYPE, "application/json")
                .status(200)
                .body(Body::from(result)),
            Err(_) => hyper::Response::builder()
                .header(header::CONTENT_TYPE, "application/json")
                .status(500)
                .body(Body::from(String::new())),
        }
    }
}
