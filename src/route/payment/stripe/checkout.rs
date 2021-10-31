use super::{SessionID, API_URL, KEY};
use actix_web::{
    client::{Client, ClientResponse},
    dev::{Decompress, Payload},
    web, HttpRequest, HttpResponse, Responder,
};
use const_format::concatcp;
use serde::{Deserialize, Serialize};

use crate::{
    database::{stripe_profile, user::User, SqlPool},
    route::{EndpointProcessingResult, BASE_URL},
};
use crate::{helper::get_current_user, route::payment::stripe::get_stripe_error_message};
use crate::{middleware, route::EndpointProcessingError};
use stripe_profile::StripeProfile;

const SUCCESS: &str = concatcp!(BASE_URL, "/payment/success");
const CANCEL: &str = concatcp!(BASE_URL, "/payment/cancel");

#[derive(Deserialize)]
pub struct CreateCheckoutRequest {
    pub price_id: String,
}

// https://stripe.com/docs/api/checkout/sessions/create
// Contains contsants for what the Stripe page should do on the frontend on failure/sucess
pub fn create_checkout_body(
    price_id: String,
    stripe_profile: StripeProfile,
) -> Result<String, serde_qs::Error> {
    #[derive(Serialize)]
    struct LineItem {
        price: String,
        quantity: usize,
    }

    #[derive(Serialize)]
    struct CreateCheckoutReqestBody<'a> {
        payment_method_types: [&'a str; 1],
        success_url: &'a str,
        cancel_url: &'a str,
        mode: &'a str,
        customer: &'a str,
        line_items: Vec<LineItem>,
    }

    serde_qs::to_string(&CreateCheckoutReqestBody {
        payment_method_types: ["card"],
        success_url: SUCCESS,
        cancel_url: CANCEL,
        mode: "subscription",
        customer: &stripe_profile.stripe_id,
        line_items: vec![LineItem {
            price: price_id,
            quantity: 1,
        }],
    })
}

// Attempt to extact the session id of a new session from a Stripe API response.
pub async fn get_session_id(
    response: ClientResponse<Decompress<Payload>>,
) -> Result<SessionID, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct StripeResponse {
        id: String,
    }

    let StripeResponse { id } = super::deserialize_stripe_res(response).await?;
    Ok(id)
}

crate::basic_error!(CheckoutCreationError, "Failed to create checkout session");

lazy_static::lazy_static! {
    pub static ref CREATE_PATH: String = format!("{}/checkout/sessions", &API_URL);
    pub static ref BEARER_KEY: String = format!("Bearer {}", KEY);
}

#[derive(Deserialize)]
pub struct StripeResponse {
    pub id: String,
    pub url: String,
}

// auth done by middleware
// Create a Checkout session on stripe and send the user the id of that session if successful.
pub async fn create_checkout(
    user: &User,
    req: CreateCheckoutRequest,
    pool: &SqlPool,
) -> EndpointProcessingResult<StripeResponse> {
    // First we auth the user
    // let user = get_current_user(&httpreq, pool)?;

    println!("[STRP] get/create stripe profile");

    // Then we make sure they have info with stripe
    let stripe_profile = stripe_profile::get_or_create(user, pool).await?;

    println!("[STRP] got stripe profile");

    // Note this is the ID of the subscription the user wants, this is assumed to be accurate since it can only be a subscription we create and publish, however checks might be needed for student status or other cases.
    let CreateCheckoutRequest { price_id } = req;
    let body = create_checkout_body(price_id, stripe_profile)?;

    let uri: &str = &CREATE_PATH;
    let key: &str = &BEARER_KEY;

    let client = Client::new();
    let res = client
        .post(uri)
        .header("Authorization", key)
        .send_body(body)
        .await
        .map_err(|e| EndpointProcessingError::Processing(Box::new(e)))?;

    if res.status().is_success() == false {
        println!("[STRP] failed to create checkout");
        return Err(get_stripe_error_message(res).await.into());
    }

    println!("[STRP] create checkout");

    Ok(super::deserialize_stripe_res(res).await?)
}

// Wrappers

// pub async fn post(
//     httpreq: HttpRequest,
//     req: web::Form<CreateCheckoutRequest>,
//     pool: web::Data<SqlPool>,
//     _: middleware::UserAuthorized,
// ) -> impl Responder {
//     use EndpointProcessingError::*;

//     match create_checkout(&httpreq, req.into_inner(), pool.as_ref()).await {
//         Ok(id) => HttpResponse::Created().body(id),
//         Err(Unauthorized) => HttpResponse::Unauthorized().finish(),
//         Err(RequestUnparsable) | Err(RequestNonsensical) => HttpResponse::BadRequest().finish(),
//         Err(e) => {
//             println!("Failed to stripe checkout with error: {}", e);
//             HttpResponse::InternalServerError().finish()
//         }
//     }
// }
