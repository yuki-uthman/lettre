//! src/tests/api/admin.rs

use crate::helpers::{assert_is_redirect_to, setup};

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    // Arrange
    let app = setup().await;

    // Act
    let response = app.get("/admin/dashboard").await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_get_change_password_form() {
    // Arrange
    let app = setup().await;

    // Act
    let response = app.get("/admin/password").await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_your_password() {
    // Arrange
    let app = setup().await;

    // Act
    let new_password_form = serde_json::json!({
        "current_password": "old password",
        "password": "new password",
        "password_confirmation": "new password",
    });

    let response = app.post_form("/admin/password", &new_password_form).await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn go_through_burp_proxy() {
    // Arrange
    let app = setup().await;

    // Act
    let proxy = reqwest::Proxy::http("http://127.0.0.1:8080").unwrap();
    let client = reqwest::Client::builder().proxy(proxy).build().unwrap();

    let response = client
        .get(&format!("{}{}", &app.address,"/admin/dashboard"))
        .send()
        .await
        .unwrap();


    // Assert
    assert_is_redirect_to(&response, "/login");
}
