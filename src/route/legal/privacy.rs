use actix_web::{web::Data, HttpResponse, Responder};

use handlebars::Handlebars;

use crate::middleware;

pub async fn get_dk(
    template: Data<Handlebars<'_>>,
    session: middleware::context::SessionContext,
) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/legal/privacy-dk", &session).unwrap())
}

pub async fn get_en(
    template: Data<Handlebars<'_>>,
    session: middleware::context::SessionContext,
) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/legal/privacy-en", &session).unwrap())
}
