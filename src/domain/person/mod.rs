//! src/domain/mod.rs
mod name;
use name::Name;

mod email;
use email::Email;

use crate::routes::SubscriberForm;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    NameError(#[from] name::Error),

    #[error(transparent)]
    EmailError(#[from] email::Error),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Person {
    pub name: Name,
    pub email: Email,
}

impl Person {
    pub fn parse(name: String, email: String) -> Result<Self, Error> {
        Ok(Self {
            name: Name::parse(name)?,
            email: Email::parse(email)?,
        })
    }
}

impl TryFrom<SubscriberForm> for Person {
    type Error = Error;

    fn try_from(form: SubscriberForm) -> Result<Self, Self::Error> {
        Ok(Self {
            email: Email::parse(form.email)?,
            name: Name::parse(form.name)?,
        })
    }
}
