//! src/routes/subscriptions.rs
use crate::{domain::Person, email::Brevo};
use actix_web::{web, HttpResponse, Result};
use chrono::Utc;
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

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
) -> HttpResponse {
    let subscriber = match Person::try_from(form.into_inner()) {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let id = match insert_subscriber(&pool, &subscriber).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let token = generate_subscription_token();
    if insert_token(&pool, id, &token).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(&email_client, &subscriber)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(name = "Sending a confirmation email", skip(email_client, subscriber))]
async fn send_confirmation_email(
    email_client: &Brevo,
    subscriber: &Person,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://127.0.0.1/subscriptions/confirm?subscription_token=token";
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
    skip(form, pool)
)]
async fn insert_subscriber(pool: &PgPool, form: &Person) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();

    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
            "#,
        id,
        form.email.as_ref(),
        form.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await?;

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
    skip(pool)
)]
async fn insert_token(pool: &PgPool, subscriber_id: Uuid, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscription_tokens (subscription_token, subscriber_id)
    VALUES ($1, $2)
            "#,
        token,
        subscriber_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
