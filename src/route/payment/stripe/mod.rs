use actix_web::{
    client::ClientResponse,
    dev::{Decompress, Payload},
};

pub use crate::keys::{STRIPE_KEY as KEY, STRIPE_WEBHOOK_SECRET as WEBHOOK_SECRET};

pub type SessionID = String;
pub const API_URL: &str = "https://api.stripe.com/v1";

crate::basic_error_with_message!(StripeError, "Stripe failed with error");

mod deserialize_stripe_res;
pub use deserialize_stripe_res::{deserialize_stripe_res, StripeAPIError};

pub async fn get_stripe_error_message(
    mut res: ClientResponse<Decompress<Payload>>,
) -> Box<dyn std::error::Error> {
    match res.json::<StripeAPIError>().await {
        Ok(body) => Box::new(body),
        Err(_) => Box::new(StripeError::static_message("Failed to read response")),
    }
}

pub mod checkout;
pub mod customer;
pub mod portal;
pub mod status;
pub mod webhook;
