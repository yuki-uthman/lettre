//! src/routes/subscriptions.rs
use crate::domain::Subscriber;
use actix_web::http::{header::ContentType, StatusCode};
use actix_web::{error::ResponseError, web, HttpResponse, Result};
use chrono::Utc;
use sqlx::PgPool;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Debug)]
enum Error {
    InternalError,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::InternalError => f.write_str("Internal Server Error"),
        }
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Error::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[allow(dead_code)]
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
) -> Result<HttpResponse> {
    insert_subscriber(&pool, &form.into())
        .await
        .map_err(|_| Error::InternalError)?;

    Ok(HttpResponse::Ok().finish())
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
