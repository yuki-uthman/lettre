//! tests/api/login.rs

use crate::helpers::setup;
use uuid::Uuid;

#[tokio::test]
async fn user_non_existing_is_rejected() {
    // Arrange
    let app = setup().await;
    // Random credentials
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    let form = [
        ("username", username.as_str()),
        ("password", password.as_str()),
    ];

    let response = reqwest::Client::new()
        .post(&format!("{}/login", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&form)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());
}
