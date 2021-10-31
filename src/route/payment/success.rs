use std::collections::HashMap;

use actix_multipart::Multipart;
use actix_web::{
    cookie::Cookie,
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
        self,
        sessions::{self, SessionToken},
        signup_tokens,
        stripe_profile::{self, get_or_create},
        user, SqlPool, SyncTransaction,
    },
    middleware,
    route::{
        payment::stripe::{
            checkout::{create_checkout, CreateCheckoutRequest},
            status::verify_stripe_subscription,
        },
        user::signup::index::SignupTierQuery,
        EndpointProcessingError, EndpointProcessingResult,
    },
};

//

pub async fn get(
    template: Data<Handlebars<'_>>,
    session: middleware::Session,
    pool: web::Data<SqlPool>,
    _: middleware::UserAuthorized,
) -> impl Responder {
    if let middleware::Session::Guest = session {
        HttpResponse::Found().header(header::LOCATION, "/").finish()
    } else {
        let user = session.unwrap_user();

        let subscription = verify_stripe_subscription(user.id, &pool)
            .await
            .unwrap()
            .unwrap();

        let stripe = match database::stripe_profile::get(user.id, pool.as_ref()) {
            Ok(Some(v)) => v,
            Err(e) => {
                println!("Failed to get verify stripe subscription with error: {}", e);
                unreachable!();
            }
            _ => unreachable!(),
        };

        stripe_profile::set_subscription(
            &stripe.stripe_id,
            subscription.current_period_start,
            subscription.current_period_end,
            &pool,
        )
        .unwrap();

        // registration and payment successful, drop the signup token
        let trans = SyncTransaction::new(&pool).unwrap();
        signup_tokens::delete(user.id, &trans).unwrap();
        trans.commit().unwrap();

        user::verify(&user, &pool).unwrap();

        HttpResponse::Ok().body(template.render("page/payment/success", &session).unwrap())
    }
}
