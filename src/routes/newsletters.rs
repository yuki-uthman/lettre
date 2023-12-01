//! src/routes/newsletters.rs
use crate::domain::Person as Subscriber;
use crate::{email::Brevo, routes::error_chain_fmt};
use actix_web::http::{
    header::{HeaderMap, HeaderValue, WWW_AUTHENTICATE},
    StatusCode,
};
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            PublishError::AuthError(_) => {
                let authentication = (
                    WWW_AUTHENTICATE,
                    HeaderValue::from_static(r#"Basic realm="publish""#),
                );

                HttpResponse::build(StatusCode::UNAUTHORIZED)
                    .insert_header(authentication)
                    .finish()
            }
        }
    }
}

#[derive(serde::Deserialize)]
pub struct Newsletter {
    title: String,
    body: String,
}

pub async fn publish(
    pool: web::Data<PgPool>,
    email_client: web::Data<Brevo>,
    payload: web::Json<Newsletter>,
) -> Result<HttpResponse, PublishError> {
    let newsletter: Newsletter = payload.into_inner();

    let confirmed_subscribers = get_confirmed_subscribers(&pool)
        .await
        .context("Failed to retrieve confirmed subscribers")?;

    let parsed_subscribers = parse_confirmed_subscribers(confirmed_subscribers);

    for subscriber in parsed_subscribers {
        let email = email_client
            .email_builder()
            .subject(&newsletter.title)
            .to(&subscriber)
            .html_content(&newsletter.body)
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

#[tracing::instrument(name = "Parse confirmed subscribers", skip(rows))]
fn parse_confirmed_subscribers(rows: Vec<Row>) -> Vec<Subscriber> {
    let mut subscribers = Vec::new();

    for row in rows {
        let result = Subscriber::parse(row.name, row.email.clone());

        match result {
            Ok(subscriber) => subscribers.push(subscriber),
            Err(e) => {
                tracing::warn!(
                    error.cause_chain = ?e,
                    "Skipping confirmed subscriber {} because {}", row.email, e);
            }
        }
    }

    subscribers
}
