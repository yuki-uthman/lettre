//! src/domain/name.rs
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("A name must not be empty")]
    Empty,
    #[error("A name must not be more than 256 graphemes long")]
    TooLong,
    #[error("A name must not contain any of the following characters: '/' '(' ')' '\"' '<' '>' '\\' '{{' '}}'")]
    InvalidCharacters,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Name(String);

impl Name {
    pub fn parse(s: String) -> Result<Self, Error> {
        let is_empty_or_whitespace = s.trim().is_empty();
        if is_empty_or_whitespace {
            return Err(Error::Empty);
        }

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two characters
        // (`a` and `̊`).
        //
        // `graphemes` returns an iterator over the graphemes in the input `s`.
        // `true` specifies that we want to use the extended grapheme definition set,
        // the recommended one.
        let is_too_long = s.graphemes(true).count() > 256;
        if is_too_long {
            return Err(Error::TooLong);
        }

        // Iterate over all characters in the input `s` to check if any of them matches
        // one of the characters in the forbidden array.
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));
        if contains_forbidden_characters {
            return Err(Error::InvalidCharacters);
        }

        Ok(Self(s))
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_ok;
    use colored::*;

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
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a̐".repeat(256);
        assert_ok!(Name::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        let result = Name::parse(name);
        matches!(result, Err(Error::TooLong));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        let result = Name::parse(name);
        matches!(result, Err(Error::Empty));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        let result = Name::parse(name);
        matches!(result, Err(Error::Empty));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();

            let result = Name::parse(name);
            matches!(result, Err(Error::InvalidCharacters));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(Name::parse(name));
    }
}
