//! src/authenticate.rs

use crate::telemetry::spawn_blocking_with_tracing;
use anyhow::Context;
use argon2::PasswordVerifier;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[derive(serde::Deserialize, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

#[tracing::instrument(name = "Authenticate user", skip(pool))]
pub async fn authenticate(
    pool: &PgPool,
    received_credentials: Credentials,
) -> Result<uuid::Uuid, AuthError> {
    let user_db = get_user(pool, &received_credentials.username)
        .await
        .context(format!(
            "Failed to retrieve user {} from the database",
            received_credentials.username
        ))
        .map_err(AuthError::UnexpectedError)?;

    let (user_id, password_hash) = match user_db {
        Some(user_db) => (Some(user_db.user_id), user_db.password_hash),
        None => {
            let hash = "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
                .to_string();

            (None, hash)
        }
    };

    spawn_blocking_with_tracing(move || {
        verify_password(received_credentials.password, password_hash)
    })
    .await
    .context("Failed to spawn blocking thread")
    .map_err(AuthError::UnexpectedError)?
    .context("Failed to verify password")
    .map_err(AuthError::InvalidCredentials)?;

    user_id
        .ok_or_else(|| anyhow::anyhow!("User not found"))
        .map_err(AuthError::InvalidCredentials)
}

struct User {
    user_id: uuid::Uuid,
    password_hash: String,

    #[allow(dead_code)]
    username: String,
}

#[tracing::instrument(name = "Get user from the database", skip(pool))]
async fn get_user(pool: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT user_id, username, password_hash
        FROM users
        WHERE username = $1
        "#,
        username,
    )
    .fetch_optional(pool)
    .await
}

#[tracing::instrument(name = "Verify password hash" skip(password, hash))]
fn verify_password(password: Secret<String>, hash: String) -> Result<(), anyhow::Error> {
    let expected_password_hash =
        argon2::PasswordHash::new(&hash).context("Failed to create password hash")?;

    argon2::Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &expected_password_hash)
        .context("Failed to verify the password")
}
