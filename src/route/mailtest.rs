use actix_web::{
    web::{self, Data},
    HttpResponse, Responder,
};

use css_inline::CSSInliner;
use handlebars::Handlebars;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Template {
    name: String,
}

pub async fn get(template: Data<Handlebars<'_>>, query: web::Query<Template>) -> impl Responder {
    let inliner = CSSInliner::compact();
    HttpResponse::Ok().body(
        inliner
            .inline(&template.render(&query.name, &()).unwrap())
            .unwrap(),
    )
}
