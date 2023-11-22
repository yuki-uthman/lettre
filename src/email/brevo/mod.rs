//! src/email/brevo/mod.rs
use crate::configuration::EmailSettings;
use crate::domain::Person;
use serde::Serialize;

mod email;
use email::{EmailBuilder, EmailClient};

mod secret;

#[derive(Debug)]
pub struct Brevo {
    sender: Person,
    email_client: EmailClient,
}

impl From<EmailSettings> for Brevo {
    fn from(email_settings: EmailSettings) -> Self {
        let name = email_settings.sender_name.clone();
        let email = email_settings.sender_email.clone();

        let sender = Person::parse(name, email).expect("Parsing person failed");

        let email_client = EmailClient::new(
            email_settings.api_url.clone(),
            email_settings.api_key.clone(),
        );

        Self::new(sender, email_client)
    }
}

impl Brevo {
    fn new(sender: Person, email_client: EmailClient) -> Self {
        Self {
            sender,
            email_client,
        }
    }

    pub fn email_builder(&self) -> EmailBuilder {
        EmailBuilder::new(&self.sender)
    }

    pub async fn send_email<T>(&self, email: &T) -> Result<(), reqwest::Error>
    where
        T: Serialize,
    {
        let _ = self.email_client.send_email(email).await;
        Ok(())
    }
}
