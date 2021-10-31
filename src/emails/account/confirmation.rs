use const_format::concatcp;
use serde::{Deserialize, Serialize};

use crate::{emails::Email, route::BASE_URL};

const SIGNUP_URL: &str = concatcp!(BASE_URL, "/payment");

/// Account registration confirmation/completion email that would redirect users to payment
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountConfirmationEmail {
    token: String,
    url: String,
}

impl AccountConfirmationEmail {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
            url: SIGNUP_URL.to_string(),
        }
    }
}

impl Email<'_> for AccountConfirmationEmail {
    fn template() -> &'static str {
        "email/account/confirmation"
    }

    fn subject() -> &'static str {
        "Orto: sign-up confirmation"
    }
}
