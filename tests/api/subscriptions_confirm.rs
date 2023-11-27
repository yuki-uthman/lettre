//! tests/api/subscriptions_confirm.rs

use crate::helpers::{extract_link_path, setup};
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

    // Assert
    let email = test.received_email().await;

    let link_path = extract_link_path(&email.html_content.as_str());

    let response = test.get(&link_path).await;
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn clicking_the_link_confirms_the_subscription() {
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

    // Assert
    let email = test.received_email().await;
    let link_path = extract_link_path(&email.html_content.as_str());
    let _ = test.get(&link_path).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&test.db_pool)
        .await
        .unwrap();

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}
