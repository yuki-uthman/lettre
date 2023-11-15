//! src/domain/subscriber_name.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
