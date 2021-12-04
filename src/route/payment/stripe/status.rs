use super::{StripeError, API_URL, KEY};
use crate::{database, route::payment::stripe::StripeAPIError};
use actix_web::{client::Client};
use database::{stripe_profile::StripeID, user::UserID, SqlPool};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Parameters {
    status: &'static str,
    customer: StripeID,
}

lazy_static::lazy_static! {
    pub static ref ENDPOINT_PATH: String = format!("{}/subscriptions", &API_URL);
    pub static ref BEARER_KEY: String = format!("Bearer {}", KEY);
}

#[derive(Deserialize, Clone, Debug)]
pub struct SubscriptionStatus {
    pub current_period_end: i64,
    pub current_period_start: i64,
    pub start_date: i64,
}

#[derive(Deserialize)]
struct SubscriptionStatusWrapper {
    data: Vec<SubscriptionStatus>,
}

// A check to see if a user has a current subscription, first by checking the database, and then Stripe
// TODO: Save the status of Stripe subscription if it differs from the database
pub async fn verify_stripe_subscription(
    user_id: UserID,
    pool: &SqlPool,
) -> Result<Option<SubscriptionStatus>, Box<dyn std::error::Error>> {
    // First we try to get the stripe profile (no profile means no subscription, not an error)
    let stripe = match database::stripe_profile::get(user_id, pool) {
        Ok(Some(v)) => v,
        Err(e) => {
            println!("Failed to get verify stripe subscription with error: {}", e);
            return Ok(None);
        }
        _ => return Ok(None),
    };

    println!("[STRP] verify: get stripe profile for user {}", user_id);

    let uri: &str = &ENDPOINT_PATH;
    let key: &str = &BEARER_KEY;

    let client = Client::default();
    let mut res = client
        .get(uri)
        .header("Authorization", key)
        .send_body(
            serde_urlencoded::to_string(&Parameters {
                status: "active",
                customer: stripe.stripe_id,
            })
            .unwrap(),
        )
        .await?;

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
    let now = crate::helper::get_millis_since_epoch()? / 1000;

    // We check if an active subscription starting before now, and ending after now exists
    println!("{:?} {}", &data, now);

    Ok(data
        .iter()
        .find(
            |SubscriptionStatus {
                 current_period_end,
                 current_period_start,
                 start_date,
             }| {
                *current_period_start <= now && *start_date <= now && *current_period_end > now
            },
        )
        .cloned())
}
