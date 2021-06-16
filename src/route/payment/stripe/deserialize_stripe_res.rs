use actix_web::{
    client::ClientResponse,
    dev::{Decompress, Payload},
};
use serde::{de, Deserialize};

#[derive(Deserialize, Debug, Clone)]
pub struct StripeAPIError {
    #[serde(rename = "type")]
    error_type: String,
    code: String,
    decline_code: Option<String>,
    message: String,
    param: String,
}

impl<'a> std::fmt::Display for StripeAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Got code {} for error type {} with message: {}",
            self.code, self.error_type, self.message
        )
    }
}

impl std::error::Error for StripeAPIError {}

pub async fn deserialize_stripe_res<T>(
    mut res: ClientResponse<Decompress<Payload>>,
) -> Result<T, Box<dyn std::error::Error>>
where
    for<'de> T: de::Deserialize<'de>,
{
    match res.body().await {
        Ok(data) => match serde_json::from_slice(&data) {
            Ok(v) => Ok(v),
            Err(_) => Err(match serde_json::from_slice::<StripeAPIError>(&data) {
                Ok(v) => Box::new(v),
                Err(e) => Box::new(e),
            }),
        },
        Err(e) => Err(Box::new(e)),
    }
}
