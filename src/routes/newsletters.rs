//! src/routes/newsletters.rs
use crate::domain::Person as Subscriber;
use crate::{email::Brevo, routes::error_chain_fmt};
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::PgPool;

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn publish(
    pool: web::Data<PgPool>,
    email_client: web::Data<Brevo>,
) -> Result<HttpResponse, PublishError> {
    let confirmed_subscribers = get_confirmed_subscribers(&pool)
        .await
        .context("Failed to retrieve confirmed subscribers")?;

    let parsed_subscribers: Vec<Subscriber> = confirmed_subscribers
        .iter()
        .map(|row| Subscriber::parse(row.name.clone(), row.email.clone()))
        .flatten()
        .collect::<Vec<_>>();

    for subscriber in parsed_subscribers {
        let body = format!(
            "Hello! This is a newsletter from your friends at The Rust Async Working Group. \
            We're happy to have you",
        );

        let email = email_client
            .email_builder()
            .subject("Newsletter")
            .to(&subscriber)
            .html_content(&body)
            .build();

        email_client
            .send_email(&email)
            .await
            .with_context(|| format!("Failed to send email to {}", subscriber.email))?;
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(serde::Deserialize)]
struct Row {
    name: String,
    email: String,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(pool: &PgPool) -> Result<Vec<Row>, sqlx::Error> {
    sqlx::query_as!(
        Row,
        r#"
        SELECT email, name
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await
}
