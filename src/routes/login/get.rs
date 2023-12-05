//! src/routes/login/get.rs
use crate::configuration::HmacSecret;
use actix_web::{http::header::ContentType, web, HttpResponse};
use hmac::{Hmac, Mac};

#[derive(serde::Deserialize, Debug)]
pub struct QueryParams {
    error: String,
    tag: Option<String>,
}

impl QueryParams {
    fn is_valid(&self, secret: &HmacSecret) -> bool {
        let tag = match &self.tag {
            Some(tag) => tag,
            None => {
                tracing::error!("Missing hmac tag in query params");
                return false;
            }
        };

        let tag = match hex::decode(tag) {
            Ok(tag) => tag,
            Err(_) => {
                tracing::error!("Invalid hex in hmac tag");
                return false;
            }
        };

        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));

        let mut mac = match Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => {
                tracing::error!("Invalid hmac secret");
                return false;
            }
        };

        mac.update(query_string.as_bytes());

        match mac.verify_slice(&tag) {
            Ok(_) => true,
            Err(_) => {
                tracing::error!("Invalid hmac tag");
                false
            }
        }
    }
}

#[tracing::instrument(name = "GET /login", skip(hmac_secret))]
pub async fn login_form(
    query_params: Option<web::Query<QueryParams>>,
    hmac_secret: web::Data<HmacSecret>,
) -> HttpResponse {
    let received_query_params = query_params.is_some();
    let query_is_valid = query_params
        .as_ref()
        .map(|qp| qp.is_valid(hmac_secret.as_ref()))
        .unwrap_or(false);

    let error_html = if received_query_params && query_is_valid {
        format!(
            "<p><i>{}</i></p>",
            htmlescape::encode_minimal(&query_params.unwrap().error)
        )
    } else {
        "".into()
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Login</title>
</head>
<body>
    {error_html}
    <form action="/login" method="post">
        <label>Username
            <input
                type="text"
                placeholder="Enter Username"
                name="username"
            >
        </label>
        <label>Password
            <input
                type="password"
                placeholder="Enter Password"
                name="password"
            >
        </label>
        <button type="submit">Login</button>
    </form>
</body>
</html>"#,
        ))
}
