//! src/email/brevo/secret.rs
use secrecy::Secret;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BrevoSecret {
    pub api_key: Secret<String>,
    pub api_url: String,
    pub sender_name: String,
    pub sender_email: String,
}
