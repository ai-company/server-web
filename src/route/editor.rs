use crate::web;
use crate::{middleware, tokens::sell_token};

use super::{Response, SharedContext};

use cookie::Cookie;
use hyper::{body::HttpBody, header, Body, Request, StatusCode};
use time::Duration;

pub async fn get(req: Request<Body>, context: &SharedContext) -> Response {
    if middleware::is_authorized(&req) {
        let template = web::get_template(&context, "editor").unwrap();
        hyper::Response::builder().body(Body::from(template))
    } else {
        let query = req.uri().query().unwrap_or_default();
        let parameters = web::parse_query(query);

        if let Some(access_token) = parameters.get("token") {
            if let Some(auth_token) = sell_token(access_token) {
                let template = web::get_template(&context, "editor");
                hyper::Response::builder()
                    .header(
                        header::SET_COOKIE,
                        Cookie::build("auth", auth_token)
                            .http_only(true)
                            .max_age(Duration::days(30))
                            .finish()
                            .encoded()
                            .to_string(),
                    )
                    .body(Body::from(template.unwrap()))
            } else {
                hyper::Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::from("No such token ID."))
            }
        } else {
            hyper::Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("Could not get token."))
        }
    }
}

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
