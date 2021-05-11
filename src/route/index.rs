use actix_web::{web, HttpRequest, HttpResponse, Responder};
use handlebars::{to_json, Handlebars};
use serde::{Deserialize, Serialize};

use crate::middleware::is_authorized;
use crate::tokens;

#[derive(Serialize)]
struct OnboardingData {
    pub authorized: bool,
}

pub async fn get(req: HttpRequest, template: web::Data<Handlebars<'_>>) -> impl Responder {
    HttpResponse::Ok().body(
        template
            .render(
                "page/index",
                &to_json(OnboardingData {
                    authorized: is_authorized(&req),
                }),
            )
            .unwrap(),
    )
}

#[derive(Deserialize)]
pub struct AlphaEmail {
    email: String,
}

pub async fn post(req: web::Form<AlphaEmail>) -> impl Responder {
    match tokens::try_send_token_to(&req.email) {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::Unauthorized(),
    }
}
