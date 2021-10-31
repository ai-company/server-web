use super::{get_stripe_error_message, API_URL, KEY};
use crate::{
    database::{stripe_profile::StripeID, user::User},
    route::payment::stripe::StripeAPIError,
};
use actix_web::client::Client;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    pub static ref STRIPE_CUSTOMER_ENDPOINT: String = format!("{}/customers", API_URL);
    pub static ref STRIPE_CUSTOMER_ENDPOINT_URI: String = format!("{}/customers", API_URL);
    pub static ref BEARER_KEY: String = format!("Bearer {}", KEY);
}

// Create a stripe account for a given customer on stripe, and if suceesful return their new Stripe id
pub async fn create_customer(user: &User) -> Result<StripeID, Box<dyn std::error::Error>> {
    #[derive(Serialize)]
    struct StripeCustomer {
        email: String,
    }

    let body_content = serde_qs::to_string(&StripeCustomer {
        email: user.email.clone(),
    })?;

    let uri: &str = &STRIPE_CUSTOMER_ENDPOINT_URI;
    let key: &str = &BEARER_KEY;

    let client = Client::default();
    let mut res = client
        .post(uri)
        .header("accept", "application/json")
        .header("authorization", key)
        .header("content-type", "application/x-www-form-urlencoded")
        // .header("host", "api.stripe.com")
        .send_body(body_content)
        // .send()
        .await?;

    if res.status().is_success() == false {
        println!(
            "[STRP] failed to create customer:\n\tres: {:?}\n\t{:?}",
            res.headers().clone(),
            &res.body().await
        );
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

// Attempt to delete a user's Stripe profile based on their ID
pub async fn delete_customer(id: &StripeID) -> Result<(), Box<dyn std::error::Error>> {
    let base: &str = &STRIPE_CUSTOMER_ENDPOINT;
    let uri: &str = &format!("{}/{}", base, id);
    let key: &str = &BEARER_KEY;

    println!("[STRP] requested customer deletion {}", id);

    let client = Client::default();
    let mut res = client
        .delete(uri)
        .header("accept", "application/json")
        .header("authorization", key)
        // .header("content-type", "application/x-www-form-urlencoded")
        // .header("host", "api.stripe.com")
        .send()
        .await?;

    if res.status().is_success() == false {
        let err = res.json::<StripeAPIError>().await.unwrap();

        // user doesn't exist
        if err.error.code == "resource_missing" {
            println!("[STRP] no customer to delete");
            return Ok(());
        }

        println!("[STRP] couldn't delete customer");
        return Err(get_stripe_error_message(res).await);
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct DeleteCustomerResponse {
        id: String,
        object: String,
        deleted: bool,
    }
    let response: DeleteCustomerResponse = super::deserialize_stripe_res(res).await?;
    if response.deleted {
        println!("[STRP] customer deleted successfully");
        Ok(())
    } else {
        println!("[STRP] couldn't delete customer");
        crate::basic_error!(StripeDeletionFailed, "Failed to delete Stripe user");
        Err(Box::new(StripeDeletionFailed))
    }
}
