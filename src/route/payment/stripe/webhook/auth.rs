use actix_web::http::HeaderMap;
use hex::encode as encode_hex;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
type HmacSha256 = Hmac<Sha256>;
use std::collections::HashMap;

use crate::route::payment::stripe::WEBHOOK_SECRET;

use crate::route::{EndpointProcessingError, EndpointProcessingResult};

// Determine the expected signature from stripe based on a shared secret, and the SHA256 hash of the request/webhook
pub fn compute_signature(payload: &str, secret: &[u8]) -> String {
    let mut mac = HmacSha256::new_varkey(secret).expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());

    let result = mac.finalize();
    let result = result.into_bytes().as_slice().to_vec();

    encode_hex(result)
}

pub fn parse_stripe_signature_header(header: &str) -> Result<HashMap<String, String>, ()> {
    let signature = header.trim().split(',');
    let signature: Vec<Vec<&str>> = signature
        .map(|pair| pair.split('=').map(|s| s.trim()).collect())
        .collect();

    let mut values: HashMap<String, String> = HashMap::new();

    for pair in &signature {
        if pair.len() != 2 {
            return Err(());
        }

        let key = pair.first();
        let value = pair.last();

        values.insert(key.unwrap().to_string(), value.unwrap().to_string());
    }

    Ok(values)
}

/// Implements Webhook payload verification, in accordance with official Stripe docs.
/// See [docs](https://stripe.com/docs/webhooks/signatures) for details.
pub fn authenticate(headers: &HeaderMap, body_content: &str) -> EndpointProcessingResult<()> {
    let signature_header = match headers.get("stripe-signature").map(|v| v.to_str()) {
        Some(Ok(v)) => v,
        _ => return Err(EndpointProcessingError::RequestUnparsable),
    };

    let parsed_header = match parse_stripe_signature_header(signature_header) {
        Ok(v) => v,
        _ => return Err(EndpointProcessingError::RequestUnparsable),
    };

    let received_timestamp = match parsed_header.get("t") {
        Some(v) => v,
        _ => return Err(EndpointProcessingError::RequestUnparsable),
    };
    let received_signature = match parsed_header.get("v1") {
        Some(v) => v,
        _ => return Err(EndpointProcessingError::RequestUnparsable),
    };

    let payload = format!("{}.{}", received_timestamp, body_content);
    let expected_signature = &compute_signature(&payload, WEBHOOK_SECRET.as_bytes());

    if expected_signature.as_bytes() == received_signature.as_bytes() {
        Ok(())
    } else {
        Err(EndpointProcessingError::Unauthorized)
    }
}
