//! src/routes/subscriptions.rs
use crate::{domain::Subscriber, email::Brevo};
use actix_web::{web, HttpResponse, Result};
use chrono::Utc;
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
    let subscriber = match Subscriber::try_from(form.into_inner()) {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let email = email_client
        .email_builder()
        .to(
            subscriber.name.as_ref().to_string(),
            subscriber.email.as_ref().to_string(),
        )
        .subject("Welcome!".to_string())
        .html_content("<p>Thanks for subscribing to our newsletter!</p>".to_string())
        .build();

    let _res = email_client.send_email(&email).await;

    match insert_subscriber(&pool, &subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
async fn insert_subscriber(pool: &PgPool, form: &Subscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        form.email.as_ref(),
        form.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await?;

    Ok(())
}
