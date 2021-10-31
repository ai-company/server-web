use serde::{Deserialize, Serialize};

use crate::emails::Email;

/// Monthly subscription billing email
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionInvoiceEmail {
    tier: String,
    total: usize,
}

impl SubscriptionInvoiceEmail {
    pub fn new(tier: &str, total: usize) -> Self {
        Self {
            tier: tier.to_string(),
            total,
        }
    }
}

impl Email<'_> for SubscriptionInvoiceEmail {
    fn template() -> &'static str {
        "email/subscription/invoice"
    }

    fn subject() -> &'static str {
        "Orto: subscription invoice"
    }
}
