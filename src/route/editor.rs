use actix_web::{web, HttpResponse, Responder};
use handlebars::Handlebars;

use crate::middleware::UserAuthorized;

pub async fn get(
    template: web::Data<Handlebars<'_>>,
    _mw_authorized: UserAuthorized,
) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/editor", &()).unwrap())
}
