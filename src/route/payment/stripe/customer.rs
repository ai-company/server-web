use super::{get_stripe_error_message, API_URL, KEY};
use crate::database::{stripe_profile::StripeID, user::User};
use actix_web::client::Client;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    pub static ref STRIPE_CUSTOMER_ENDPOINT: String = format!("{}/customers", API_URL);
    pub static ref STRIPE_CUSTOMER_ENDPOINT_URI: String = format!("{}/customers", API_URL);
    pub static ref BEARER_KEY: String = format!("Bearer {}", KEY);
}

pub async fn create_customer(user: User) -> Result<StripeID, Box<dyn std::error::Error>> {
    #[derive(Serialize)]
    struct StripeCustomer {
        email: String,
    }

    let body_content = serde_qs::to_string(&StripeCustomer { email: user.email })?;

    let uri: &str = &STRIPE_CUSTOMER_ENDPOINT_URI;
    let key: &str = &BEARER_KEY;

    let client = Client::default();
    let res = client
        .post(uri)
        .header("accept", "application/json")
        .header("authorization", key)
        .header("content-type", "application/x-www-form-urlencoded")
        .header("host", "api.stripe.com")
        .send_body(body_content)
        .await?;

    if res.status().is_success() == false {
        return Err(get_stripe_error_message(res).await);
    }

    #[derive(Deserialize)]
    struct StripeCustomerResponse {
        id: String,
    }

    Ok(super::deserialize_stripe_res::<StripeCustomerResponse>(res)
        .await?
        .id)
}

pub async fn delete_customer(id: &StripeID) -> Result<(), Box<dyn std::error::Error>> {
    let base: &str = &STRIPE_CUSTOMER_ENDPOINT;
    let uri: &str = &format!("{}/{}", base, id);
    let key: &str = &BEARER_KEY;

    let client = Client::default();
    let res = client
        .post(uri)
        .header("accept", "application/json")
        .header("authorization", key)
        .header("content-type", "application/x-www-form-urlencoded")
        .header("host", "api.stripe.com")
        .send()
        .await?;

    if res.status().is_success() == false {
        return Err(get_stripe_error_message(res).await);
    }

    #[derive(Deserialize)]
    struct DeleteCustomerResponse {
        // id: String,
        // object: String,
        deleted: bool,
    }
    let response: DeleteCustomerResponse = super::deserialize_stripe_res(res).await?;
    if response.deleted {
        Ok(())
    } else {
        crate::basic_error!(StripeDeletionFailed, "Failed to delete Stripe user");
        Err(Box::new(StripeDeletionFailed))
    }
}
