//! src/email/brevo/mod.rs
use crate::domain::Person;
use reqwest::Client;
use serde::Serialize;

mod email;
use email::{EmailBuilder, EmailClient};

mod secret;
use secret::BrevoSecret;

#[derive(Debug)]
pub struct Brevo {
    sender: Person,
    email_client: EmailClient,
}

impl Brevo {
    pub fn with_secret(filename: &str) -> Self {
        let brevo_secret = BrevoSecret::from_filename(filename);

        let name = brevo_secret.sender_name.clone();
        let email = brevo_secret.sender_email.clone();

        let sender = Person::parse(name, email).expect("Parsing person failed");

        let email_client = EmailClient {
            http_client: Client::new(),
            url: brevo_secret.api_url.clone(),
            api_key: brevo_secret.api_key.clone(),
        };

        Self::new(sender, email_client)
    }

    fn new(sender: Person, email_client: EmailClient) -> Self {
        Self {
            sender,
            email_client,
        }
    }

    pub fn email_builder(&self) -> EmailBuilder {
        EmailBuilder::new(self.sender.clone())
    }

    pub async fn send_email<T>(&self, email: &T) -> Result<(), reqwest::Error>
    where
        T: Serialize,
    {
        let _ = self.email_client.send_email(email).await;
        Ok(())
    }
}
