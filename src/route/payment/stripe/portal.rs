use super::{API_URL, KEY};
use actix_web::{
    client::{Client, ClientResponse},
    dev::{Decompress, Payload},
    web, HttpRequest, HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};

use crate::{
    database::{stripe_profile, SqlPool},
    helper::get_current_user,
    middleware,
};
use stripe_profile::StripeID;

fn create_portal_body(stripe_id: StripeID, return_url: &str) -> serde_json::Result<String> {
    #[derive(Serialize)]
    struct CreateCheckoutReqestBody<'a> {
        customer: &'a str,
        return_url: &'a str,
    }

    serde_json::to_string(&CreateCheckoutReqestBody {
        customer: &stripe_id,
        return_url,
    })
}

async fn get_session_url(
    response: ClientResponse<Decompress<Payload>>,
) -> Result<PortalID, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct StripeResponse {
        url: String,
    }

    let StripeResponse { url } = super::deserialize_stripe_res(response).await?;
    Ok(url)
}

pub type PortalID = String;

lazy_static::lazy_static! {
    pub static ref CREATE_PATH: String = format!("{}/billing_portal/sessions/", &API_URL);
}

const RETURN_URL: &str = "lol idk";

crate::basic_error!(PortalCreationError, "Failed to register protal with stripe");

async fn create_portal(
    req: &HttpRequest,
    pool: &SqlPool,
) -> Result<PortalID, Box<dyn std::error::Error>> {
    // First we auth the user
    let user = get_current_user(req, pool)?;

    // Then we make sure they have info with stripe
    let stripe_profile = stripe_profile::get_or_create(&user, pool).await?;

    let body = create_portal_body(stripe_profile.stripe_id, RETURN_URL)?;

    let uri: &str = &CREATE_PATH;

    let client = Client::default();
    let req = client
        .post(uri)
        .header("authorization", KEY)
        .send_body(body);

    match req.await {
        Ok(r) => get_session_url(r).await,
        Err(e) => {
            println!(
                "Failed to create checkout session on Stripe API with error: {}",
                e
            );
            Err(Box::new(PortalCreationError))
        }
    }
}

// Wrappers

pub async fn post(
    req: HttpRequest,
    pool: web::Data<SqlPool>,
    _: middleware::UserAuthorized,
) -> impl Responder {
    match create_portal(&req, pool.as_ref()).await {
        Ok(id) => HttpResponse::Created().body(id),
        Err(e) => {
            println!("Failed to stripe portal with error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
