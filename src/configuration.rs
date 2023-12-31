//! src/configuration.rs
use config::{Config, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email: Option<EmailSettings>,
}

impl Settings {
    pub fn set_email_url(&mut self, email_url: String) {
        if let Some(email_settings) = &mut self.email {
            email_settings.api_url = email_url;
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct HmacSecret(pub Secret<String>);

impl HmacSecret {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.expose_secret().as_bytes()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub redis_uri: Secret<String>,
    pub hmac_secret: Option<HmacSecret>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct EmailSettings {
    pub api_key: Secret<String>,
    pub api_url: String,
    pub sender_name: String,
    pub sender_email: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    /// Omitting the database name connects to the Postgres instance, not a specific logical database.
    /// This is useful for operations that create or drop databases.
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

#[derive(PartialEq)]
pub enum Environment {
    Local,
    Production,
}
impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_ref() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            _ => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                s
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    // Detect the running environment.
    // Default to `local` if not specified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let settings = Config::builder()
        .add_source(File::from(configuration_directory.join("base")).required(true))
        .add_source(File::from(configuration_directory.join(environment.as_str())).required(true))
        .build()?;

    let mut settings: Settings = settings.try_deserialize()?;

    if environment == Environment::Local {
        let secret_file_path = configuration_directory.join("secrets");
        dotenvy::from_filename(secret_file_path).expect("Failed to read secrets file");
    }

    let email_settings = envy::prefixed("EMAIL_CLIENT_")
        .from_env::<EmailSettings>()
        .expect("Failed to parse email settings from environment");
    settings.email = Some(email_settings);

    let hmac_secret = std::env::var("HMAC_SECRET").expect("HMAC_SECRET must be set");
    settings.application.hmac_secret = Some(HmacSecret(Secret::new(hmac_secret)));

    Ok(settings)
}
