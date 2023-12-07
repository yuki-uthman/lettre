//! tests/api/login.rs

use crate::helpers::{assert_is_redirect_to, setup};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let app = setup().await;

    // Random credentials
    let response = app.login("fake user", "fake password").await;

    // Assert
    assert_is_redirect_to(&response, "/login");

    let html_page = app.get_text("/login").await;
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));

    // Act - Part 3 - Reload the login page
    let html_page = app.get_text("/login").await;
    assert!(!html_page.contains("<p><i>Authentication failed</i></p>"));
}

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    // Arrange
    let app = setup().await;
    let username = &app.user.username;
    let password = &app.user.password;

    // Act
    let response = app.login(username, password).await;

    // Assert
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_text("/admin/dashboard").await;
    assert!(html_page.contains(&format!("Welcome {}", app.user.username)));
}
