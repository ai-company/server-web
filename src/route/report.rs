use crate::middleware;

use super::{Response, SharedContext};

use hyper::{body::HttpBody, header, Body, Client, Request};

pub async fn post(req: Request<Body>, _context: &SharedContext) -> Response {
    if middleware::is_authorized(&req) {
        let client = Client::new();

        let body_vector: Vec<u8> = req.into_body().data().await.unwrap().unwrap().to_vec();
        let body_string = String::from_utf8(body_vector).unwrap();

        client
            .request(
                hyper::Request::builder()
                    .method("POST")
                    .uri("http://127.0.0.1:6969/api/report/")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(body_string))
                    .expect("Builder(Bob) not building right."),
            )
            .await;

        hyper::Response::builder().body(Body::from("Give me back my feet."))
    } else {
        hyper::Response::builder().body(Body::from(""))
    }
}
