#[tokio::test]
async fn health_check_works() {
    // Start the application as a background task
    spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() {
    let server = lettre::run().expect("Failed to spawn our app.");

    // Launch the server as a background task
    let _ = tokio::spawn(server);
}
