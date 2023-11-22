//! tests/api/helpers.rs

use letter::configuration::get_configuration;
use letter::email::Brevo;
use letter::telemetry::{get_subscriber, init_subscriber};
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

pub struct TestSetup {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn setup() -> TestSetup {
    Lazy::force(&TRACING);

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
    let db_pool = PgPool::connect(config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database");

    let email_client = Brevo::from(config.email.unwrap());

    let localhost = "127.0.0.1";

    // binding to port 0 will trigger an OS scan for an available port
    let tcp_listener =
        TcpListener::bind(format!("{}:0", localhost)).expect("Failed to bind random port");

    let assigned_port = tcp_listener.local_addr().unwrap().port();

    let server = letter::startup::run(tcp_listener, db_pool.to_owned(), email_client)
        .expect("Failed to bind address");

    // Launch the server as a background task
    let _ = tokio::spawn(server);

    let address = format!("http://{}:{}", localhost, assigned_port);

    TestSetup { address, db_pool }
}
