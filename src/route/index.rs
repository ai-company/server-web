use actix_web::{web::Data, HttpResponse, Responder};

use handlebars::Handlebars;

use crate::middleware;

pub async fn get(
    template: Data<Handlebars<'_>>,
    session: middleware::context::SessionContext,
) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/index", &session).unwrap())
}
