use actix_web::{web, HttpResponse, Responder};
use handlebars::Handlebars;

use crate::middleware::{Session, UserAuthorized};

pub async fn get(
    template: web::Data<Handlebars<'_>>,
    _mw_authorized: UserAuthorized,
    session: Session,
) -> impl Responder {
    HttpResponse::Ok().body(template.render("page/editor", &session).unwrap())
}
