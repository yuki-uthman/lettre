//! src/routes/subscriptions_confirm.rs

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber" skip(pool))]
pub async fn confirm(
    pool: web::Data<sqlx::PgPool>,
    params: web::Query<Parameters>,
) -> HttpResponse {
    tracing::info!("{:#?}", params);
    let result = get_subscriber_id(&pool, &params.subscription_token).await;

    let subscriber_id = match result {
        Ok(Some(subscriber_id)) => subscriber_id,
        Ok(None) => return HttpResponse::BadRequest().finish(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let result = confirm_subscriber(&pool, subscriber_id).await;

    if result.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

async fn get_subscriber_id(
    pool: &sqlx::PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT subscriber_id
        FROM subscription_tokens
        WHERE subscription_token = $1
        "#,
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map(|row| row.map(|r| r.subscriber_id))
}

async fn confirm_subscriber(pool: &sqlx::PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions
        SET status = 'confirmed'
        WHERE id = $1 AND status = 'pending_confirmation'
        "#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map(|_| ())
}
