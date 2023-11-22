//! src/email/email.rs
use crate::domain::Person;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Email<'a> {
    sender: &'a Person,
    pub to: Vec<&'a Person>,
    pub subject: &'a str,
    #[serde(rename = "htmlContent")]
    pub html_content: &'a str,
}

pub struct EmailBuilder<'a> {
    sender: &'a Person,
    to: Vec<&'a Person>,
    subject: &'a str,
    html_content: &'a str,
}

impl<'a> EmailBuilder<'a> {
    pub fn new(sender: &'a Person) -> Self {
        Self {
            sender,
            to: vec![],
            subject: "",
            html_content: "",
        }
    }

    pub fn to(mut self, person: &'a Person) -> Self {
        self.to.push(person);
        self
    }

    pub fn subject(mut self, subject: &'a str) -> Self {
        self.subject = subject;
        self
    }

    pub fn html_content(mut self, html_content: &'a str) -> Self {
        self.html_content = html_content;
        self
    }

    pub fn build(self) -> Email<'a> {
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
    pub async fn send_email<T>(&self, email: &T) -> Result<reqwest::Response, reqwest::Error>
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
            .error_for_status()?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_makes_http_request() {
        // Arrange
        let mock_server = MockServer::start().await;

        let api_key = Secret::new(Faker.fake::<String>());

        let name = Faker.fake::<String>();
        let email = SafeEmail().fake::<String>();
        let sender = Person::parse(name, email).expect("Parsing person failed");

        let client = EmailClient {
            http_client: Client::new(),
            url: mock_server.uri(),
            api_key,
        };

        let recipient =
            Person::parse(Faker.fake::<String>(), SafeEmail().fake::<String>()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let html_content = Paragraph(1..10).fake::<String>();
        let email = EmailBuilder::new(&sender)
            .to(&recipient)
            .subject(&subject)
            .html_content(&html_content)
            .build();

        Mock::given(method("POST"))
            .and(path("/"))
            .and(header_exists("api-key"))
            .and(header("accept", "application/json"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = client.send_email(&email).await;
    }

    #[tokio::test]
    async fn send_email_returns_ok_with_200_response() {
        // Arrange
        let mock_server = MockServer::start().await;

        let api_key = Secret::new(Faker.fake::<String>());

        let name = Faker.fake::<String>();
        let email = SafeEmail().fake::<String>();
        let sender = Person::parse(name, email).expect("Parsing person failed");

        let client = EmailClient {
            http_client: Client::new(),
            url: mock_server.uri(),
            api_key,
        };

        let recipient =
            Person::parse(Faker.fake::<String>(), SafeEmail().fake::<String>()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let html_content = Paragraph(1..10).fake::<String>();
        let email = EmailBuilder::new(&sender)
            .to(&recipient)
            .subject(&subject)
            .html_content(&html_content)
            .build();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let result = client.send_email(&email).await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_returns_err_with_error_response() {
        // Arrange
        let mock_server = MockServer::start().await;

        let api_key = Secret::new(Faker.fake::<String>());

        let name = Faker.fake::<String>();
        let email = SafeEmail().fake::<String>();
        let sender = Person::parse(name, email).expect("Parsing person failed");

        let client = EmailClient {
            http_client: Client::new(),
            url: mock_server.uri(),
            api_key,
        };

        let recipient =
            Person::parse(Faker.fake::<String>(), SafeEmail().fake::<String>()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let html_content = Paragraph(1..10).fake::<String>();
        let email = EmailBuilder::new(&sender)
            .to(&recipient)
            .subject(&subject)
            .html_content(&html_content)
            .build();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let result = client.send_email(&email).await;
        assert_err!(result);
    }
}
