//! src/routes/login/post.rs
use crate::{
    authenticate::{validate_credentials, AuthError, Credentials},
    configuration::HmacSecret,
    routes::error_chain_fmt,
};
use actix_web::{error::InternalError, web, HttpResponse};
use hmac::{Hmac, Mac};
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
    hmac_secret: web::Data<HmacSecret>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = form.into_inner();
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, pool.as_ref()).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

            Ok(HttpResponse::SeeOther()
                .insert_header(("Location", "/"))
                .finish())
        }
        Err(error) => {
            let error = match error {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(error.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(error.into()),
            };

            let encoded_error = urlencoding::Encoded::new(error.to_string());
            let query_string = format!("error={}", encoded_error);

            let hmac_tag = {
                let mut mac =
                    Hmac::<sha2::Sha256>::new_from_slice(hmac_secret.into_inner().as_bytes())
                        .unwrap();
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };

            let redirect_url = format!("/login?{}&tag={:x}", query_string, hmac_tag);
            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, redirect_url))
                .finish();

            Err(InternalError::from_response(error, response))
        }
    }
}
