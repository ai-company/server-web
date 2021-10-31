//! System emails
//!
//! This module defines a handlebars based email templating system
//!
//! Example email object definition:
//! ```
//! use serde::{Deserialize, Serialize};
//! use crate::emails::Email;
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! pub struct MyEmail {
//!     thing: String,
//! }
//!
//! impl MyEmail {
//!     pub fn new(thing: String) -> Self {
//!         Self { thing }
//!     }
//! }
//!
//! impl Email<'_> for MyEmail {
//!     fn template() -> &'static str {
//!         "email/example"
//!     }
//!
//!     fn subject() -> &'static str {
//!         "Orto: example email"
//!     }
//! }
//! ```
//!
//! Example usage:
//! ```
//! MyEmail::new("hello".into()).render(&handlebars).send("someone@example.com")
//! ```
//!

pub mod account;
pub mod subscription;

use std::collections::HashMap;

use css_inline::CSSInliner;
use handlebars::to_json;
use lettre::smtp::error::SmtpResult;
use serde::{Deserialize, Serialize};

use crate::email;

pub trait Email<'e>: Serialize + Deserialize<'e> {
    fn template() -> &'static str;
    fn subject() -> &'static str;

    fn variables(&self) -> handlebars::JsonValue {
        to_json(self)
    }

    fn render(self, renderer: &handlebars::Handlebars) -> Envelope {
        let inliner = CSSInliner::compact();

        Envelope(
            Self::subject().to_string(),
            inliner
                .inline(
                    &renderer
                        .render(Self::template(), &self.variables())
                        .unwrap(),
                )
                .unwrap(),
        )
    }
}

pub struct Envelope(String, String);

impl Envelope {
    pub fn send(self, to: &str) -> SmtpResult {
        email::email_send(to, &self.0, &self.1)
    }
}
