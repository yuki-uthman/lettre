//! src/domain/subscriber_email.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
