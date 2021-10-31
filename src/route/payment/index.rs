use std::collections::HashMap;

use actix_multipart::Multipart;
use actix_web::{
    cookie::{Cookie, SameSite},
    http::header,
    web::{self, Data},
    HttpResponse, Responder,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::EnumVariantNames;

use futures::stream::{Stream, StreamExt};
use handlebars::Handlebars;
use serde_json::json;
use time::Duration;

use crate::{
    database::{
        sessions::{self, SessionToken},
        signup_tokens,
        stripe_profile::get_or_create,
        user, SqlPool, SyncTransaction,
    },
    middleware,
    route::{
        payment::stripe::checkout::{create_checkout, CreateCheckoutRequest},
        user::signup::index::SignupTierQuery,
        EndpointProcessingError, EndpointProcessingResult,
    },
};

const TIER_MAP: [(&'static str, &'static str); 3] = [
    ("student", "price_1IC9WtLKbmf1O8QW4FgBWQS6"),
    ("private", "price_1IC9WtLKbmf1O8QW4FgBWQS6"),
    ("business", "price_1IC9WtLKbmf1O8QW4FgBWQS6"),
];

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationQuery {
    token: Option<String>,
}

pub async fn get(
    template: Data<Handlebars<'_>>,
    token: web::Query<RegistrationQuery>,
    session: middleware::Session,
    pool: web::Data<SqlPool>,
) -> impl Responder {
    let pool = pool.into_inner();

    if let middleware::Session::Guest = session {
        let user_id = match signup_tokens::get_owner(&pool, &token.token.as_ref().unwrap()) {
            Ok(Some(user_id)) => user_id,
            Ok(_) => {
                return HttpResponse::BadRequest()
                    .body(template.render("page/user/signup/no_id", &session).unwrap())
            }
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(template.render("page/user/signup/no_id", &session).unwrap())
            }
        };

        let user = match user::get_with_id(user_id, &pool) {
            Ok(Some(user)) => user,
            Ok(_) | Err(_) => {
                return HttpResponse::InternalServerError()
                    .body(template.render("page/user/signup/no_id", &session).unwrap())
            }
        };

        let stripe_response = match create_checkout(
            &user,
            CreateCheckoutRequest {
                price_id: TIER_MAP
                    .iter()
                    .find(|i| i.0 == user.tier.as_ref().unwrap())
                    .unwrap()
                    .1
                    .into(),
            },
            &pool,
        )
        .await
        {
            Ok(id) => id,
            Err(e) => {
                println!("Failed to pay with error: {}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

        let session_token = sessions::generate(&pool, user_id).unwrap();

        HttpResponse::Found()
            .cookie(
                Cookie::build("session", &session_token)
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .max_age(Duration::days(30))
                    .finish(),
            )
            .header(header::LOCATION, stripe_response.url)
            .finish()
    } else {
        let user = session.unwrap_user();

        let stripe_response = match create_checkout(
            &user,
            CreateCheckoutRequest {
                price_id: TIER_MAP
                    .iter()
                    .find(|i| i.0 == user.tier.as_ref().unwrap())
                    .unwrap()
                    .1
                    .into(),
            },
            &pool,
        )
        .await
        {
            Ok(id) => id,
            Err(e) => {
                println!("Failed to pay with error: {}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

        HttpResponse::Found()
            .header(header::LOCATION, stripe_response.url)
            .finish()
    }
}
