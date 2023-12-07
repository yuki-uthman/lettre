//! tests/api/subscriptions.rs

use crate::helpers::{extract_link_path, setup};
use wiremock::{
    matchers::{any, method},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test = setup().await;

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = test.post_body("/subscriptions", body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let test = setup().await;

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let _ = test.post_body("/subscriptions", body.into()).await;

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_sends_email_for_valid_form_data() {
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
    let response = test.post_body("/subscriptions", body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_sends_email_with_a_link() {
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
    let _ = test.post_body("/subscriptions", body.into()).await;

    // Assert
    let email = test.received_email().await;

    extract_link_path(&email.html_content);
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let test = setup().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, error_message) in test_cases {
        // Act
        let response = test.post_body("/subscriptions", body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_name_is_invalid() {
    // Arrange
    let test = setup().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        (
            "name=bracket%7B&email=ursula_le_guin%40gmail.com",
            "contains invalid char",
        ),
    ];

    for (body, error_message) in test_cases {
        // Act
        let response = test.post_body("/subscriptions", body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_email_is_invalid() {
    // Arrange
    let test = setup().await;
    let test_cases = vec![
        ("name=le%20guin&email=", "empty email"),
        ("name=le%20guin&email=notanemail", "invalid email"),
    ];

    for (body, error_message) in test_cases {
        // Act
        let response = test.post_body("/subscriptions", body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_fails_if_sth_wrong_with_subscriptions_table() {
    // Arrange
    let app = setup().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Sabotage the database
    sqlx::query!("ALTER TABLE subscriptions DROP COLUMN email;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.post_body("/subscriptions", body.into()).await;

    // Assert
    assert_eq!(response.status().as_u16(), 500);
}

#[tokio::test]
async fn subscribe_fails_if_sth_wrong_with_subscriptions_token_table() {
    // Arrange
    let app = setup().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Sabotage the database
    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.post_body("/subscriptions", body.into()).await;

    // Assert
    assert_eq!(response.status().as_u16(), 500);
}

#[tokio::test]
async fn subscribe_fails_if_sth_wrong_with_name() {
    // Arrange
    let app = setup().await;
    let body = "name=le%20guin%7B&email=ursula_le_guin@gmail.com";

    // Act
    let response = app.post_body("/subscriptions", body.into()).await;

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn subscribe_fails_if_sth_wrong_with_email() {
    // Arrange
    let app = setup().await;
    let body = "name=le%20guin&email=ursula_le_guin";

    // Act
    let response = app.post_body("/subscriptions", body.into()).await;

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}
