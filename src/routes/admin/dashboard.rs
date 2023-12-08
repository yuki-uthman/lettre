//! src/routes/admin/dashboard.rs

use crate::session_state::TypedSession;
use crate::utils::e500;
use actix_web::{http::header::ContentType, web, HttpResponse};
use anyhow::Context;
use uuid::Uuid;

pub async fn admin_dashboard(
    session: TypedSession,
    pool: web::Data<sqlx::PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id_from_session = session.get_user_id().map_err(e500)?;

    if user_id_from_session.is_none() {
        return Ok(HttpResponse::SeeOther()
            .insert_header(("Location", "/login"))
            .finish());
    }

    let user_id = user_id_from_session.unwrap();
    let username = get_username(user_id, pool.get_ref()).await.map_err(e500)?;

    let body = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Admin dashboard</title>
</head>
<body>
    <p>Welcome {username}!</p>
</body>
</html>"#
    );

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}

async fn get_username(user_id: Uuid, pool: &sqlx::PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to query for username")?;

    Ok(row.username)
}
