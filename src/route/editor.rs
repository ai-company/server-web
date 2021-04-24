use crate::web;
use crate::{middleware, tokens::sell_token};

use crate::route::{Response, SharedContext};

use cookie::Cookie;
use hyper::{header, Body, Request, StatusCode};
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
