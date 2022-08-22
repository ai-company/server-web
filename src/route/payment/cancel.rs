use actix_web::{http::header, web::Data, HttpResponse, Responder};

use handlebars::Handlebars;

use crate::middleware;

//

pub async fn get(
    template: Data<Handlebars<'_>>,
    session: middleware::context::SessionContext,
    _: middleware::UserAuthorized,
) -> impl Responder {
    if let middleware::Session::Guest = session.user {
        HttpResponse::Found().header(header::LOCATION, "/").finish()
    } else {
        HttpResponse::Ok().body(template.render("page/payment/cancel", &session).unwrap())
    }
}
