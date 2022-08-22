use std::{collections::HashMap, fmt::Debug};

use actix_web::{
    cookie::{Cookie, SameSite},
    error::InternalError,
    http::header,
    web::{self, Data},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};

use serde::{Deserialize, Serialize};
use strum::EnumVariantNames;

use handlebars::Handlebars;
use serde_json::json;
use time::Duration;

use crate::{
    database::{
        billing_info, sessions::SessionToken, signup_tokens, user, SqlPool, SyncTransaction,
    },
    emails::{account::confirmation::AccountConfirmationEmail, newsletter::NewsletterEmail, Email},
    middleware,
    route::EndpointProcessingError,
    util::FormData,
    validator::{v, Validator, ValidatorResult},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewsletterRequest {
    pub message: String,
}

// wrappers

pub async fn post(
    req: FormData,
    pool: web::Data<SqlPool>,
    template: Data<Handlebars<'_>>,
    _: middleware::AdminAuthorized,
) -> impl Responder {
    let req = req.parse::<NewsletterRequest>().unwrap();

    match user::get_all(&pool) {
        Ok(users) => {
            let emails = users.iter().map(|u| u.email.as_ref()).collect::<Vec<_>>();

            match NewsletterEmail::new(&req.message)
                .render(&template)
                .send_all(&emails)
            {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => HttpResponse::InternalServerError().body(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(json!({
            "error": format!("[ db ] could not get users {}", e)
        })),
    }
}
