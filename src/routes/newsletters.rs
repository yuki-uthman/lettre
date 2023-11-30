//! src/routes/newsletters.rs
use actix_web::HttpResponse;

pub async fn publish() -> HttpResponse {
    HttpResponse::Ok().finish()
}
