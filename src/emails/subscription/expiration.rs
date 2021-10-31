use serde::{Deserialize, Serialize};

use crate::emails::Email;

/// Subscription expiration warning email
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionExpirationEmail {
    tier: String,
    total: usize,
}

impl SubscriptionExpirationEmail {
    pub fn new(tier: &str, total: usize) -> Self {
        Self {
            tier: tier.to_string(),
            total,
        }
    }
}

impl Email<'_> for SubscriptionExpirationEmail {
    fn template() -> &'static str {
        "email/subscription/expiration"
    }

    fn subject() -> &'static str {
        "Orto: subscription expiration notice"
    }
}
