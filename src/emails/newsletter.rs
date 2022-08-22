use crate::emails::Email;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewsletterEmail {
    message: String,
    date: String,
}

impl NewsletterEmail {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            date: chrono::offset::Utc::today().format("%Y-%m-%d").to_string(),
        }
    }
}

impl Email<'_> for NewsletterEmail {
    fn template() -> &'static str {
        "email/newsletter"
    }

    fn subject() -> &'static str {
        "Orto: newsletter"
    }
}
