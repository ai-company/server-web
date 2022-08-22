use actix_web::Responder;
use actix_web::{web, HttpResponse};
use handlebars::Handlebars;

use crate::middleware;

pub async fn get(
    template: web::Data<Handlebars<'_>>,
    session: middleware::context::SessionContext,
) -> impl Responder {
    HttpResponse::NotFound().body(template.render("page/404", &session).unwrap())
}
