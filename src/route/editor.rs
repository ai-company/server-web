use actix_web::{web, HttpResponse, Responder};
use handlebars::Handlebars;

use crate::middleware;

pub async fn get(
    template: web::Data<Handlebars<'_>>,
    session: middleware::context::SessionContext,
    _: middleware::UserAuthorized,
) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/editor", &session).unwrap())
}
