use actix_web::{client::Client, http::header, web::Data, HttpRequest, HttpResponse, Responder};

use crate::middleware;
use crate::route::SharedContext;

pub async fn post(
    req: HttpRequest,
    body_string: String,
    _context: Data<SharedContext>,
) -> impl Responder {
    if middleware::is_authorized(&req) {
        let client = Client::new();

        client
            .post("http://127.0.0.1:6969/api/report")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .send_body(body_string)
            .await;

        HttpResponse::Ok().body("Give me back my feet.")
    } else {
        HttpResponse::Ok().body("")
    }
}
