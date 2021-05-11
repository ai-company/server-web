use actix_web::Responder;
use actix_web::{web, HttpResponse};
use handlebars::Handlebars;

pub async fn get(template: web::Data<Handlebars<'_>>) -> impl Responder {
    HttpResponse::NotFound().body(template.render("page/404", &()).unwrap())
}
