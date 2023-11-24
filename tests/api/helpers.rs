//! tests/api/helpers.rs

use letter::configuration::get_configuration;
use letter::startup::build;
use letter::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use reqwest::Url;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;

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

pub struct Test {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl Test {
    pub async fn get(&self, path: &str) -> reqwest::Response {
        reqwest::get(&format!("{}{}", self.address, path))
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post(&self, path: &str, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}{}", self.address, path))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn setup() -> Test {
    Lazy::force(&TRACING);

    let mut config = get_configuration().expect("Failed to read configuration.");
    config.application.port = 0;
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

    // Start email server
    let email_server = MockServer::start().await;
    config.set_email_url(email_server.uri());

    // Launch the server
    let app = build(config.clone()).expect("Failed to build server.");
    let address = format!("http://127.0.0.1:{}", app.port());
    config.application.port = app.port();

    tracing::info!("Test running with the following Settings:\n{:#?}", config);

    // Launch the server as a background task
    let _ = tokio::spawn(app.run());

    Test {
        address,
        db_pool,
        email_server,
    }
}

pub fn extract_link_path(s: &str) -> Option<String> {
    let links: Vec<_> = linkify::LinkFinder::new()
        .links(s)
        .filter(|link| *link.kind() == linkify::LinkKind::Url)
        .collect();

    if links.len() == 0 {
        return None;
    }

    let url = Url::parse(links[0].as_str()).expect("Failed to parse link.");

    if let Some(query) = url.query() {
        Some(format!("{}?{}", url.path(), query))
    } else {
        Some(url.path().to_string())
    }
}
