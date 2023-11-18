//! src/email/brevo/mod.rs
use reqwest::Client;
use serde::Serialize;

mod email;
use email::{EmailBuilder, EmailClient, Person};

mod secret;
use secret::BrevoSecret;

pub struct Brevo {
    pub sender: Person,
    pub email_client: EmailClient,
}

impl Brevo {
    pub fn with_secret(filename: &str) -> Self {
        let brevo_secret = BrevoSecret::from_filename(filename);

        let sender = Person {
            name: brevo_secret.sender_name.clone(),
            email: brevo_secret.sender_email.clone(),
        };

        let email_client = EmailClient {
            http_client: Client::new(),
            url: brevo_secret.api_url.clone(),
            api_key: brevo_secret.api_key.clone(),
        };

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
