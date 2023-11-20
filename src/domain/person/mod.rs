//! src/domain/mod.rs
mod name;
use name::Name;

mod email;
use email::Email;

use crate::routes::SubscriberForm;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Person {
    pub name: Name,
    pub email: Email,
}

impl TryFrom<SubscriberForm> for Person {
    type Error = String;

    fn try_from(form: SubscriberForm) -> Result<Self, Self::Error> {
        Ok(Self {
            email: Email::parse(form.email)?,
            name: Name::parse(form.name)?,
        })
    }
}
