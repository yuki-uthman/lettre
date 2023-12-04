//! src/routes/login/post.rs
use crate::{
    authenticate::{validate_credentials, AuthError, Credentials},
    routes::error_chain_fmt,
};
use actix_web::{web, HttpResponse, ResponseError};
use reqwest::{header::LOCATION, StatusCode};
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

impl ResponseError for LoginError {
    fn error_response(&self) -> HttpResponse {
        let encoded_error = urlencoding::Encoded::new(self.to_string());
        HttpResponse::build(self.status_code())
            .insert_header((LOCATION, format!("/login?error={}", encoded_error)))
            .finish()
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::SEE_OTHER
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
) -> Result<HttpResponse, LoginError> {
    let credentials = form.into_inner();
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, pool.as_ref()).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

            Ok(HttpResponse::SeeOther()
                .insert_header(("Location", "/"))
                .finish())
        }
        Err(error) => match error {
            AuthError::InvalidCredentials(_) => Err(LoginError::AuthError(error.into())),
            AuthError::UnexpectedError(_) => Err(LoginError::UnexpectedError(error.into())),
        },
    }
}
