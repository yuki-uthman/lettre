//! src/routes/subscriptions_confirm.rs

use actix_web::{web, HttpResponse};
use serde::Deserialize;

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
    let subscription_token = params.into_inner().subscription_token;

    let pool = pool.get_ref();
    let result = sqlx::query!(
        r#"
        SELECT subscriber_id
        FROM subscription_tokens
        WHERE subscription_token = $1
        "#,
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map(|row| row.map(|r| r.subscriber_id));

    let subscriber_id = match result {
        Ok(Some(subscriber_id)) => subscriber_id,
        Ok(None) => return HttpResponse::BadRequest().finish(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let result = sqlx::query!(
        r#"
        UPDATE subscriptions
        SET status = 'confirmed'
        WHERE id = $1 AND status = 'pending_confirmation'
        RETURNING id
        "#,
        subscriber_id
    )
    .fetch_one(pool)
    .await;

    if result.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}
