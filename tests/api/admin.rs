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
async fn you_must_be_logged_in_to_change_password() {
    // Arrange
    let app = setup().await;

    // Act
    let response = app.get("/admin/password").await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}
