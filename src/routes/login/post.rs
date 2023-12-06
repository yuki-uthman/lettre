//! src/routes/login/post.rs
use crate::{
    authenticate::{validate_credentials, AuthError, Credentials},
    routes::error_chain_fmt,
};
use actix_web::{error::InternalError, web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use reqwest::header::LOCATION;
use sqlx::PgPool;

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[tracing::instrument(
    name = "Login a user",
    skip(form, pool),
    fields(
        username=tracing::field::Empty,
        user_id=tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<Credentials>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = form.into_inner();
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, pool.as_ref()).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

            Ok(HttpResponse::SeeOther()
                .insert_header(("Location", "/admin/dashboard"))
                .finish())
        }
        Err(error) => {
            let error = match error {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(error.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(error.into()),
            };

            FlashMessage::error(error.to_string()).send();
            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                .finish();

            Err(InternalError::from_response(error, response))
        }
    }
}
