use actix_web::{web::Data, HttpResponse, Responder};

use handlebars::Handlebars;

pub async fn get(template: Data<Handlebars<'_>>) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/index", &()).unwrap())
}
