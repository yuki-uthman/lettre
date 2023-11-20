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

#[cfg(test)]
mod test {
    use super::*;
    use colored::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::Name as FakeName;
    use fake::Fake;

    macro_rules! matches {
        ($expression:expr, $($pattern:tt)+) => {
            match $expression {
                $($pattern)+ => (),
                ref e => {
                    let right = stringify!($($pattern)+).green();
                    let left = format!("{:?}", e).red();
                    println!();
                    println!("     {} =! {}", left, right);
                    println!();
                    panic!();
                },
            }
        }
    }

    #[test]
    fn valid_name_and_email() {
        let name: String = FakeName().fake();
        let email: String = SafeEmail().fake();

        let result = Person::parse(name, email);
        matches!(result, Ok(_));
    }

    #[test]
    fn empty_name_is_rejected() {
        let name = "".to_string();
        let email = SafeEmail().fake();

        matches!(Person::parse(name, email), Err(Error::NameError(_)));
    }

    #[test]
    fn invalid_name_is_rejected() {
        let name = "{hello}".to_string();
        let email = SafeEmail().fake();

        matches!(Person::parse(name, email), Err(Error::NameError(_)));
    }

    #[test]
    fn empty_email_is_rejected() {
        let name = FakeName().fake();
        let email = "".to_string();

        matches!(Person::parse(name, email), Err(Error::EmailError(_)));
    }

    #[test]
    fn invalid_email_is_rejected() {
        let name = FakeName().fake();
        let email = "this is not an email".to_string();

        matches!(Person::parse(name, email), Err(Error::EmailError(_)));
    }
}
