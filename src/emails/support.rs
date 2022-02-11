use crate::emails::Email;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SupportEmail {
    comment: String,
    email: String,
}

impl SupportEmail {
    pub fn new(email: &str, comment: &str) -> Self {
        Self {
            email: email.to_string(),
            comment: comment.to_string(),
        }
    }
}

impl Email<'_> for SupportEmail {
    fn template() -> &'static str {
        "email/support"
    }

    fn subject() -> &'static str {
        "Orto: support/feedback"
    }
}
