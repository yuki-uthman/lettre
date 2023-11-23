//! tests/api/subscriptions_confirm.rs

use crate::helpers::setup;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let test = setup().await;

    // Act
    let response = test.get("/subscriptions/confirm").await;

    // Assert
    assert_eq!(400, response.status().as_u16());
}
