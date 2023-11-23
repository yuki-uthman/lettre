//! src/routes/subscriptions_confirm.rs

use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_params))]
pub async fn confirm(_params: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
