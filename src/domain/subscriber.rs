//! src/domain/subscriber.rs
use crate::{
    domain::{SubscriberEmail, SubscriberName},
    routes::SubscriberForm,
};
use actix_web::web::Form;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Subscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl TryFrom<Form<SubscriberForm>> for Subscriber {
    type Error = String;

    fn try_from(form: Form<SubscriberForm>) -> Result<Self, Self::Error> {
        Ok(Self {
            email: SubscriberEmail::parse(form.email.to_owned())?,
            name: SubscriberName::parse(form.name.to_owned())?,
        })
    }
}
