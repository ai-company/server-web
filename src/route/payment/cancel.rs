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
        sessions::{self, SessionToken},
        stripe_profile::get_or_create,
        user, SqlPool,
    },
    middleware,
    route::{
        payment::stripe::checkout::{create_checkout, CreateCheckoutRequest},
        user::signup::index::SignupTierQuery,
        EndpointProcessingError, EndpointProcessingResult,
    },
};

//

pub async fn get(
    template: Data<Handlebars<'_>>,
    session: middleware::Session,
    _: middleware::UserAuthorized,
) -> impl Responder {
    if let middleware::Session::Guest = session {
        HttpResponse::Found().header(header::LOCATION, "/").finish()
    } else {
        HttpResponse::Ok().body(template.render("page/payment/cancel", &session).unwrap())
    }
}
