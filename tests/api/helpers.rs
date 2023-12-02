//! tests/api/helpers.rs

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
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
    pub user: User,
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

    pub async fn post_newsletter(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", self.address))
            .header("Content-Type", "application/json")
            .basic_auth(&self.user.username, Some(&self.user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn received_email(&self) -> Email {
        let email_request = self.email_server.received_requests().await.unwrap();
        let email_request = if email_request.len() == 1 {
            &email_request[0]
        } else {
            panic!(
                "Expected 1 email to be sent but instead {} were sent.",
                email_request.len()
            );
        };

        let email: Email =
            serde_json::from_slice(&email_request.body).expect("Failed to parse email");

        email
    }
}

async fn add_user(db_pool: &PgPool, user: &User) {
    let salt = SaltString::generate(&mut rand::thread_rng());

    let password_hash = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(user.password.as_bytes(), &salt)
    .unwrap()
    .to_string();

    sqlx::query!(
        "INSERT INTO users (user_id, username, password_hash) VALUES ($1, $2, $3)",
        user.user_id,
        user.username,
        password_hash,
    )
    .execute(db_pool)
    .await
    .expect("Failed to create test users");
}

pub struct User {
    user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl User {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
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

    // Create test admin user
    let user = User::generate();
    add_user(&db_pool, &user).await;

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
        user,
    }
}

pub fn extract_link_path(s: &str) -> String {
    let links: Vec<_> = linkify::LinkFinder::new()
        .links(s)
        .filter(|link| *link.kind() == linkify::LinkKind::Url)
        .collect();

    if links.len() == 0 {
        panic!("No links found in email.");
    }

    let url = Url::parse(links[0].as_str()).expect("Failed to parse link.");

    if let Some(query) = url.query() {
        format!("{}?{}", url.path(), query)
    } else {
        url.path().to_string()
    }
}

#[derive(serde::Deserialize)]
pub struct Email {
    #[serde(rename = "htmlContent")]
    pub html_content: String,
}
