//! src/routes/subscriptions.rs
use crate::domain::{self, Person};
use crate::email::Brevo;
use actix_web::{web, HttpResponse, ResponseError, Result};
use chrono::Utc;
use rand::Rng;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug)]
pub enum SubscribeError {
    ParseError(domain::person::Error),
    DatabaseError(sqlx::Error),
    SendEmailError(reqwest::Error),
}
impl std::fmt::Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to create a new subscriber.")
    }
}

impl std::error::Error for SubscribeError {}

impl ResponseError for SubscribeError {}

impl From<domain::person::Error> for SubscribeError {
    fn from(e: domain::person::Error) -> Self {
        Self::ParseError(e)
    }
}

impl From<reqwest::Error> for SubscribeError {
    fn from(e: reqwest::Error) -> Self {
        Self::SendEmailError(e)
    }
}

impl From<sqlx::Error> for SubscribeError {
    fn from(e: sqlx::Error) -> Self {
        Self::DatabaseError(e)
    }
}

#[derive(serde::Deserialize)]
pub struct SubscriberForm {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<SubscriberForm>,
    pool: web::Data<PgPool>,
    email_client: web::Data<Brevo>,
) -> Result<HttpResponse, SubscribeError> {
    let subscriber = Person::try_from(form.into_inner())?;

    let mut transaction = pool.begin().await?;

    let id = insert_subscriber(&mut transaction, &subscriber).await?;

    let token = generate_subscription_token();
    insert_token(&mut transaction, id, &token).await?;

    transaction.commit().await?;

    send_confirmation_email(&email_client, &subscriber, &token).await?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Sending a confirmation email", skip(email_client, subscriber))]
async fn send_confirmation_email(
    email_client: &Brevo,
    subscriber: &Person,
    token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "http://127.0.0.1/subscriptions/confirm?subscription_token={}",
        token
    );
    let html_content = format!(
        "<p>Thanks for subscribing to our newsletter!</p><br/>Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    let email = email_client
        .email_builder()
        .to(subscriber)
        .subject("Welcome!")
        .html_content(&html_content)
        .build();

    email_client.send_email(&email).await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, transaction)
)]
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    form: &Person,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();

    let query = sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
            "#,
        id,
        form.email.as_ref(),
        form.name.as_ref(),
        Utc::now()
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:#?}", e);
        e
    })?;

    Ok(id)
}

fn generate_subscription_token() -> String {
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .take(20)
        .collect()
}

#[tracing::instrument(
    name = "Saving new subscription token in the database",
    skip(transaction)
)]
async fn insert_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    token: &str,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
    INSERT INTO subscription_tokens (subscription_token, subscriber_id)
    VALUES ($1, $2)
            "#,
        token,
        subscriber_id
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:#?}", e);
        e
    })?;

    Ok(())
}
