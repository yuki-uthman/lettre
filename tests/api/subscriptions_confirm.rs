//! tests/api/subscriptions_confirm.rs

use crate::helpers::setup;
use reqwest::Url;
use wiremock::{
    matchers::{any, method},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let test = setup().await;

    // Act
    let response = test.get("/subscriptions/confirm").await;

    // Assert
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn link_returns_a_200_if_clicked() {
    // Arrange
    let test = setup().await;

    Mock::given(any())
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test.email_server)
        .await;

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let _ = test.post("/subscriptions", body.into()).await;

    #[derive(serde::Deserialize, Debug)]
    struct Email {
        #[serde(rename = "htmlContent")]
        html_content: String,
    }

    // Assert
    let email_request = &test.email_server.received_requests().await.unwrap();
    if email_request.len() != 1 {
        panic!("Expected 1 email request, got {}", email_request.len(),);
    }

    let email: Email = serde_json::from_slice(&email_request[0].body).unwrap();

    let get_link = |s: &str| -> Result<String, Box<dyn std::error::Error>> {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|link| *link.kind() == linkify::LinkKind::Url)
            .collect();
        if links.len() != 1 {
            panic!("Error parsing for link: {:#?}", s);
        }

        let url = Url::parse(links[0].as_str())?;
        let link = format!(
            "{}?{}",
            url.path(),
            url.query().unwrap_or_default().to_owned()
        );
        Ok(link)
    };

    let link = get_link(&email.html_content.as_str()).unwrap();

    let response = test.get(&link).await;
    assert_eq!(200, response.status().as_u16());
}
