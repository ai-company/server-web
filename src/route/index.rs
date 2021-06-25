use actix_web::{web::Data, HttpResponse, Responder};

use handlebars::Handlebars;
use serde_json::json;

use crate::middleware::Session;

pub async fn get(template: Data<Handlebars<'_>>, session: Session) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/index", &session).unwrap())
}
