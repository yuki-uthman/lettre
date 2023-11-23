//! src/routes/subscriptions.rs
use crate::{domain::Person, email::Brevo};
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
    let subscriber = match Person::try_from(form.into_inner()) {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber(&pool, &subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    let email = email_client
        .email_builder()
        .to(&subscriber)
        .subject("Welcome!")
        .html_content("<p>Thanks for subscribing to our newsletter!</p>")
        .build();

    if email_client.send_email(&email).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
async fn insert_subscriber(pool: &PgPool, form: &Person) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'confirmed')
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
