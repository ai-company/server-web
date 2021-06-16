use actix_web::{client::Client, http::header, web::Data, HttpResponse, Responder};

use crate::{middleware::UserAuthorized, route::SharedContext};

pub async fn post(
    body_string: String,
    _context: Data<SharedContext>,
    _mw_authorized: UserAuthorized,
) -> impl Responder {
    let client = Client::new();

    client
        .post("http://127.0.0.1:6969/api/report")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send_body(body_string)
        .await;

    HttpResponse::Ok().body("Give me back my feet.")
}
