//! tests/health_check.rs
use lettre::configuration::get_configuration;
use lettre::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    // Set TEST_LOG=true to see logs during tests
    // Use bunyan to format the logs nicely:
    // $ TEST_LOG=true cargo test| bunyan
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

/// Spin up an instance of our application
/// and returns its address (i.e. http://localhost:XXXX)
fn spawn_app(pool: PgPool) -> String {
    Lazy::force(&TRACING);

    let localhost = "127.0.0.1";

    // binding to port 0 will trigger an OS scan for an available port
    let tcp_listener =
        TcpListener::bind(format!("{}:0", localhost)).expect("Failed to bind random port");

    let assigned_port = tcp_listener.local_addr().unwrap().port();

    let server = lettre::startup::run(tcp_listener, pool).expect("Failed to bind address");

    // Launch the server as a background task
    let _ = tokio::spawn(server);

    format!("http://{}:{}", localhost, assigned_port)
}

async fn setup_database() -> PgPool {
    let mut config = get_configuration().expect("Failed to read configuration.");
    config.database.database_name = Uuid::new_v4().to_string();

    // Create database
    let mut connection = PgConnection::connect(
        config
            .database
            .connection_string_without_db()
            .expose_secret(),
    )
    .await
    .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(
            r#"CREATE DATABASE "{}";"#,
            config.database.database_name
        ))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect(config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    let conn = setup_database().await;
    let address = spawn_app(conn);

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let conn = setup_database().await;
    let app_address = spawn_app(conn.clone());

    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&conn)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let conn = setup_database().await;
    let app_address = spawn_app(conn);
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
