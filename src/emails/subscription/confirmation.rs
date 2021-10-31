use serde::{Deserialize, Serialize};

use crate::emails::Email;

/// Subscription confirmation email
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionConfirmationEmail {
    tier: String,
    price: usize,
    tax: usize,
    total: usize,
}

impl SubscriptionConfirmationEmail {
    pub fn new(tier: &str, price: usize, tax: usize) -> Self {
        Self {
            tier: tier.to_string(),
            price,
            tax,
            total: price + tax,
        }
    }
}

impl Email<'_> for SubscriptionConfirmationEmail {
    fn template() -> &'static str {
        "email/subscription/confirmation"
    }

    fn subject() -> &'static str {
        "Orto: subscription confirmation"
    }
}
