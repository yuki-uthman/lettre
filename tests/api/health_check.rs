//! tests/api/health_check.rs

use crate::helpers::setup;

#[tokio::test]
async fn health_check_works() {
    let test = setup().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &test.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
