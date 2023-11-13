//! tests/health_check.rs
use std::net::TcpListener;

/// Spin up an instance of our application
/// and returns its address (i.e. http://localhost:XXXX)
fn spawn_app() -> String {
    let localhost = "127.0.0.1";

    // binding to port 0 will trigger an OS scan for an available port
    let tcp_listener =
        TcpListener::bind(format!("{}:0", localhost)).expect("Failed to bind random port");

    let assigned_port = tcp_listener.local_addr().unwrap().port();

    let server = lettre::run(tcp_listener).expect("Failed to bind address");

    // Launch the server as a background task
    let _ = tokio::spawn(server);

    format!("http://{}:{}", localhost, assigned_port)
}

#[tokio::test]
async fn health_check_works() {
    // Start the application as a background task
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
