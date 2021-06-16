use super::{SessionID, API_URL, KEY};
use actix_web::{
    client::{Client, ClientResponse},
    dev::{Decompress, Payload},
    web, HttpRequest, HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};

use crate::{
    database::{stripe_profile, SqlPool},
    route::EndpointProcessingResult,
};
use crate::{helper::get_current_user, route::payment::stripe::get_stripe_error_message};
use crate::{middleware::UserAuthorized, route::EndpointProcessingError};
use stripe_profile::StripeProfile;

const SUCCESS: &str = "https://google.com";
const CANCEL: &str = "https://bing.com";

#[derive(Deserialize)]
pub struct CreateCheckoutRequest {
    pub price_id: String,
}

// https://stripe.com/docs/api/checkout/sessions/create

fn create_checkout_body(
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

async fn get_session_id(
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

// auth done by middleware
async fn create_checkout(
    httpreq: &HttpRequest,
    req: CreateCheckoutRequest,
    pool: &SqlPool,
) -> EndpointProcessingResult<SessionID> {
    // First we auth the user
    let user = get_current_user(&httpreq, pool)?;

    // Then we make sure they have info with stripe
    let stripe_profile = stripe_profile::get_or_create(user, pool).await?;

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
        return Err(get_stripe_error_message(res).await.into());
    }

    Ok(get_session_id(res).await?)
}

// Wrappers

pub async fn post(
    httpreq: HttpRequest,
    req: web::Form<CreateCheckoutRequest>,
    pool: web::Data<SqlPool>,
    _mw_authorized: UserAuthorized,
) -> impl Responder {
    use EndpointProcessingError::*;

    match create_checkout(&httpreq, req.into_inner(), pool.as_ref()).await {
        Ok(id) => HttpResponse::Created().body(id),
        Err(Unauthorized) => HttpResponse::Unauthorized().finish(),
        Err(RequestUnparsable) | Err(RequestNonsensical) => HttpResponse::BadRequest().finish(),
        Err(e) => {
            println!("Failed to stripe checkout with error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
