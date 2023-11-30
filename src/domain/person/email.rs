//! src/domain/email.rs
use serde::{Deserialize, Serialize};
use validator::validate_email;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Empty email")]
    Empty,
    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Email(String);

impl Email {
    pub fn parse(s: String) -> Result<Self, Error> {
        if s.is_empty() {
            return Err(Error::Empty);
        }

        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(Error::Invalid(format!("Invalid email: {}", s)))
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use colored::*;
    use fake::faker::internet::en::SafeEmail;
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
    fn empty_string_is_rejected() {
        let email = "".to_string();
        let result = Email::parse(email);
        matches!(result, Err(Error::Empty));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        let result = Email::parse(email);
        matches!(result, Err(Error::Invalid(_)));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        let result = Email::parse(email);
        matches!(result, Err(Error::Invalid(_)));
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        Email::parse(valid_email.0).is_ok()
    }
}
