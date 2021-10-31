use serde::{Deserialize, Serialize};

use crate::emails::Email;

/// Account deletion email
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountDeletionEmail;

impl AccountDeletionEmail {
    pub fn new() -> Self {
        Self
    }
}

impl Email<'_> for AccountDeletionEmail {
    fn template() -> &'static str {
        "email/account/deletion"
    }

    fn subject() -> &'static str {
        "Orto: account deletion"
    }
}
