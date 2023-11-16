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

impl From<Form<SubscriberForm>> for Subscriber {
    fn from(form: Form<SubscriberForm>) -> Self {
        Self {
            email: SubscriberEmail::parse(form.email.to_owned()),
            name: SubscriberName::parse(form.name.to_owned())
                .expect("Error parsing subscriber name"),
        }
    }
}
