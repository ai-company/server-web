use actix_web::Responder;
use actix_web::{web, HttpResponse};

use crate::util;

use crate::route::SharedContext;

pub async fn get(context: web::Data<SharedContext>) -> impl Responder {
    let template = util::get_template(&context, "404").unwrap();

    HttpResponse::NotFound().body(template)
}
