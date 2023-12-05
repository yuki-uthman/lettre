//! tests/api/login.rs

use crate::helpers::{setup, assert_is_redirect_to};
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

    let response = app.post_login(&form).await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}
