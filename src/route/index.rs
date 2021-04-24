use crate::middleware::is_authorized;
use crate::tokens;
use crate::web;

use crate::route::{Response, SharedContext};

use crate::urldecode;
use hyper::{body::HttpBody, header, Body, Request, StatusCode};

pub async fn get(req: Request<Body>, context: &SharedContext) -> Response {
    let template = web::get_template(
        &context,
        if is_authorized(&req) {
            "index-authorized"
        } else {
            "index"
        },
    )
    .unwrap();

    hyper::Response::builder().body(Body::from(template))
}

pub async fn post(req: Request<Body>, _context: &SharedContext) -> Response {
    if is_authorized(&req) {
        hyper::Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, "/editor")
            .body(Body::from(""))
    } else {
        let body_vector: Vec<u8> = req.into_body().data().await.unwrap().unwrap().to_vec();
        let body_string = String::from_utf8(body_vector).unwrap();
        let form_data = web::parse_form(&body_string);

        match tokens::try_send_token_to(&urldecode(form_data.get("email").unwrap().clone())) {
            Ok(_) => hyper::Response::builder().body(Body::from("")),
            Err(_) => hyper::Response::builder().status(401).body(Body::from("")),
        }
    }
}
