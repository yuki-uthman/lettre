//! tests/api/login.rs

use crate::helpers::{assert_is_redirect_to, setup};
use uuid::Uuid;

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
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

    match response.cookies().find(|c| c.name() == "_flash") {
        Some(flash_cookie) => {
            assert_eq!(flash_cookie.value(), "Authentication failed");
        }
        None => panic!("Flash cookie not found"),
    };
}
