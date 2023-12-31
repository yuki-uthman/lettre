//! tests/api/newsletters.rs

use crate::helpers::{extract_link_path, setup, Email, Test};
use uuid::Uuid;
use wiremock::{
    matchers::{any, method},
    Mock, ResponseTemplate,
};

/// Use the public API of the application under test to create
/// an unconfirmed subscriber.
async fn create_unconfirmed_subscriber(app: &Test) -> Email {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(any())
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_body("/subscriptions", body.into()).await;

    app.received_email().await
}

async fn create_confirmed_subscriber(app: &Test) {
    let email = create_unconfirmed_subscriber(app).await;
    let confirmation_link = extract_link_path(&email.html_content);
    app.get(&confirmation_link).await;
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = setup().await;
    create_unconfirmed_subscriber(&app).await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // We assert that no request is fired at Postmark!
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act
    // A sketch of the newsletter payload structure. // We might change it later on.
    let newsletter_request_body = serde_json::json!({
             "title": "Newsletter title",
             "body": "Newsletter body",
    });
    let response = app.post_newsletter(newsletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // Mock verifies on Drop that we haven't sent the newsletter email
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = setup().await;
    create_confirmed_subscriber(&app).await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // We assert that no request is fired at Postmark!
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    // A sketch of the newsletter payload structure. // We might change it later on.
    let newsletter_request_body = serde_json::json!({
             "title": "Newsletter title",
             "body": "Newsletter body",
    });
    let response = app.post_newsletter(newsletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // Mock verifies on Drop that we haven't sent the newsletter email
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = setup().await;

    let missing_title = serde_json::json!({
        "body": "Newsletter body",
    });
    let missing_content = serde_json::json!({
        "title": "Newsletter title",
    });
    let test_cases = vec![
        (missing_title, "missing title"),
        (missing_content, "missing content"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletter(invalid_body).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    // Arrange
    let app = setup().await;

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "body": "Newsletter body",
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn user_non_existing_is_rejected() {
    // Arrange
    let app = setup().await;
    // Random credentials
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "body": "Newsletter body",
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn user_invalid_password_is_rejected() {
    // Arrange
    let app = setup().await;
    let username = &app.user.username;
    // Random password
    let password = Uuid::new_v4().to_string();
    assert_ne!(app.user.password, password);

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "body": "Newsletter body",
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}
