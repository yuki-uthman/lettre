//! src/routes/subscriptions.rs
use actix_web::{web, HttpResponse, Responder};

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct Subscriber {
    email: String,
    name: String,
}

pub async fn subscribe(_form: web::Form<Subscriber>) -> impl Responder {
    HttpResponse::Ok()
}
