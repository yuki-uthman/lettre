//! src/email/email.rs
use crate::domain::Person;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;
use std::time::Duration;

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
    http_client: Client,
    url: String,
    api_key: Secret<String>,
}

impl EmailClient {
    pub fn new(url: String, api_key: Secret<String>) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build reqwest::Client");

        Self {
            http_client,
            url,
            api_key,
        }
    }

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

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn person() -> Person {
        Person::parse(Faker.fake(), SafeEmail().fake()).expect("Parsing person failed")
    }

    fn email_client(uri: &str) -> EmailClient {
        EmailClient::new(uri.to_string(), Secret::new(Faker.fake::<String>()))
    }

    #[tokio::test]
    async fn send_email_makes_http_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let client = email_client(&mock_server.uri());

        let sender = person();
        let recipient = person();
        let subject = subject();
        let html_content = content();
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
        let client = email_client(&mock_server.uri());

        let sender = person();
        let recipient = person();
        let subject = subject();
        let html_content = content();
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
        let client = email_client(&mock_server.uri());

        let sender = person();
        let recipient = person();
        let subject = subject();
        let html_content = content();
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

    #[tokio::test]
    async fn send_email_returns_err_if_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let client = email_client(&mock_server.uri());

        let sender = person();
        let recipient = person();
        let subject = subject();
        let html_content = content();
        let email = EmailBuilder::new(&sender)
            .to(&recipient)
            .subject(&subject)
            .html_content(&html_content)
            .build();

        let response = ResponseTemplate::new(200).set_delay(Duration::from_secs(10));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let result = client.send_email(&email).await;
        assert_err!(result);
    }
}
