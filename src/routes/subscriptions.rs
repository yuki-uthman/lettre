//! src/routes/subscriptions.rs
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
pub struct Subscriber {
    email: String,
    name: String,
}

pub async fn subscribe(
    form: web::Form<Subscriber>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
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
    .await
    .map_err(|e| {
        println!("Failed to execute query: {}", e);
        Error::InternalError
    })?;

    Ok(HttpResponse::Ok().finish())
}
