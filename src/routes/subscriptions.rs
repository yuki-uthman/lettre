//! src/routes/subscriptions.rs
use actix_web::http::{header::ContentType, StatusCode};
use actix_web::{error::ResponseError, web, HttpResponse, Result};
use chrono::Utc;
use sqlx::PgPool;
use std::fmt::Display;
use tracing::Instrument;
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
pub struct Subscriber {
    email: String,
    name: String,
}

pub async fn subscribe(
    form: web::Form<Subscriber>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "New subscriber!",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name= %form.name
    );
    let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Saving new subscriber details in the database");

    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.as_ref())
    .instrument(query_span)
    .await
    .map_err(|e| {
        tracing::error!("{}: Failed to execute query:\n{:#?}", request_id, e);
        Error::InternalError
    })?;

    tracing::info!(
        "{}: Saved new subscriber details in the database.",
        request_id
    );
    Ok(HttpResponse::Ok().finish())
}
