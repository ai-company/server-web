use super::{StripeError, API_URL, KEY};
use crate::{database, route::payment::stripe::StripeAPIError};
use actix_web::client::Client;
use database::{stripe_profile::StripeID, user::UserID, SqlPool};
use serde::{Deserialize, Serialize};

fn make_query_params(stripe_id: StripeID) -> Result<String, serde_qs::Error> {
    #[derive(Serialize)]
    struct Parameters {
        status: [&'static str; 1],
        customer: StripeID,
    }

    serde_qs::to_string(&Parameters {
        status: ["active"],
        customer: stripe_id,
    })
}

lazy_static::lazy_static! {
    pub static ref ENDPOINT_PATH: String = format!("{}/subscriptions", &API_URL);
    pub static ref BEARER_KEY: String = format!("Bearer {}", KEY);
}

#[derive(Deserialize)]
struct SubscriptionStatus {
    current_period_end: i64,
    current_period_start: i64,
    start_date: i64,
}

#[derive(Deserialize)]
struct SubscriptionStatusWrapper {
    data: Vec<SubscriptionStatus>,
}

// A check to see if a user has a current subscription, first by checking the database, and then Stripe
// TODO: Save the status of Stripe subscription if it differs from the database
pub async fn verify_stripe_subscription(
    user_id: UserID,
    pool: SqlPool,
) -> Result<bool, Box<dyn std::error::Error>> {
    // First we try to get the stripe profile (no profile means no subscription, not an error)
    let stripe = match database::stripe_profile::get(user_id, &pool) {
        Ok(Some(v)) => v,
        Err(e) => {
            println!("Failed to get verify stripe subscription with error: {}", e);
            return Ok(false);
        }
        _ => return Ok(false),
    };

    let base: &str = &ENDPOINT_PATH;
    let uri = format!("{}/{}", base, make_query_params(stripe.stripe_id)?);
    let key: &str = &BEARER_KEY;

    let client = Client::default();
    let mut res = client.post(uri).header("Authorization", key).send().await?;

    if !res.status().is_success() {
        return Err(match res.json::<StripeAPIError>().await {
            Ok(body) => Box::new(body),
            Err(_) => Box::new(StripeError::static_message("Failed to read response")),
        });
    }

    // Otherwise we parse the JSON to get the current subscription data
    let SubscriptionStatusWrapper { data } = match res.body().await {
        Ok(data) => match serde_json::from_slice(&data) {
            Ok(v) => Ok(v),
            Err(_) => Err(match serde_json::from_slice::<StripeAPIError>(&data) {
                Ok(v) => Box::new(v),
                Err(e) => return Err(Box::new(e)),
            }),
        },
        Err(e) => return Err(Box::new(e)),
    }?;
    let now = crate::helper::get_millis_since_epoch()?;

    // We check if an active subscription starting before now, and ending after now exists

    Ok(data.iter().any(
        |SubscriptionStatus {
             current_period_end,
             current_period_start,
             start_date,
         }| *current_period_start <= now && *start_date <= now && *current_period_end > now,
    ))
}
