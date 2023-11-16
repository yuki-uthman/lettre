//! src/domain/subscriber.rs
use crate::{
    domain::{SubscriberEmail, SubscriberName},
    routes::SubscriberForm,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Subscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl TryFrom<SubscriberForm> for Subscriber {
    type Error = String;

    fn try_from(form: SubscriberForm) -> Result<Self, Self::Error> {
        Ok(Self {
            email: SubscriberEmail::parse(form.email)?,
            name: SubscriberName::parse(form.name)?,
        })
    }
}
