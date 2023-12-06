//! src/startup.rs
use crate::configuration::{HmacSecret, Settings};
use crate::email::Brevo;
use crate::routes::newsletters;
use crate::routes::{confirm, health_check, home, login, login_form, subscribe};
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run(self) -> std::io::Result<()> {
        self.server.await
    }
}

pub fn build(config: Settings) -> Result<Application, std::io::Error> {
    let address = format!("127.0.0.1:{}", config.application.port);
    let tcp_listener = TcpListener::bind(address).expect("Failed to bind port");
    let port = tcp_listener.local_addr().unwrap().port();
    let connection = PgPool::connect_lazy(config.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres.");

    let email_client = Brevo::from(config.email.unwrap());

    let hmac_secret = config.application.hmac_secret.expect("Missing HMAC secret");

    let server = run(tcp_listener, connection, email_client, hmac_secret)?;

    Ok(Application { port, server })
}

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: Brevo,
    hmac_secret: HmacSecret,
) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let email_client = web::Data::new(email_client);
    let hmac_secret = web::Data::new(hmac_secret);

    let message_store =
        CookieMessageStore::builder(Key::from(hmac_secret.0.expose_secret().as_bytes())).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(message_framework.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(newsletters::publish))
            // serving HTML files
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .app_data(connection.clone())
            .app_data(email_client.clone())
            .app_data(hmac_secret.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
