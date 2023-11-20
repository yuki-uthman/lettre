//! src/email/email.rs
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct Email {
    sender: Person,
    pub to: Vec<Person>,
    pub subject: String,
    #[serde(rename = "htmlContent")]
    pub html_content: String,
}

#[derive(Default)]
pub struct EmailBuilder {
    sender: Person,
    to: Vec<Person>,
    subject: String,
    html_content: String,
}

impl EmailBuilder {
    pub fn new(sender: Person) -> Self {
        Self {
            sender,
            ..Default::default()
        }
    }

    pub fn to(mut self, name: String, email: String) -> Self {
        let person = Person { name, email };

        self.to.push(person);
        self
    }

    pub fn subject(mut self, subject: String) -> Self {
        self.subject = subject;
        self
    }

    pub fn html_content(mut self, html_content: String) -> Self {
        self.html_content = html_content;
        self
    }

    pub fn build(self) -> Email {
        Email {
            sender: self.sender,
            to: self.to,
            subject: self.subject,
            html_content: self.html_content,
        }
    }
}

#[derive(Debug)]
pub struct EmailClient {
    pub http_client: Client,
    pub url: String,
    pub api_key: Secret<String>,
}

impl EmailClient {
    pub async fn send_email<T>(&self, email: &T) -> Result<serde_json::Value, reqwest::Error>
    where
        T: Serialize,
    {
        let res = self
            .http_client
            .post(&self.url)
            .header("api-key", self.api_key.expose_secret())
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .json(&email)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_makes_http_request() {
        // Arrange
        let mock_server = MockServer::start().await;

        let api_key = Secret::new(Faker.fake::<String>());
        let sender = Person {
            name: Faker.fake::<String>(),
            email: SafeEmail().fake::<String>(),
        };
        let client = EmailClient {
            http_client: Client::new(),
            url: mock_server.uri(),
            api_key,
        };

        let subject = Sentence(1..2).fake::<String>();
        let html_content = Paragraph(1..10).fake::<String>();
        let email = EmailBuilder::new(sender.clone())
            .to(Faker.fake::<String>(), SafeEmail().fake::<String>())
            .subject(subject.clone())
            .html_content(html_content.clone())
            .build();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = client.send_email(&email).await;
    }
}
