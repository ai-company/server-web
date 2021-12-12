use const_format::concatcp;
use serde::{Deserialize, Serialize};

use crate::{emails::Email, route::BASE_URL};

const RESET_URL: &str = concatcp!(BASE_URL, "/user/password_reset");

/// Password reset email
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountPasswordResetEmail {
    token: String,
    url: String,
}

impl AccountPasswordResetEmail {
    pub fn new(token: &str) -> Self {
        Self {
            url: RESET_URL.to_string(),
            token: token.to_string(),
        }
    }
}

impl Email<'_> for AccountPasswordResetEmail {
    fn template() -> &'static str {
        "email/account/password_reset"
    }

    fn subject() -> &'static str {
        "Orto - Genoprettelse af password."
    }
}
