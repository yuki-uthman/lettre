//! src/startup.rs
use crate::configuration::Settings;
use crate::email::Brevo;
use crate::routes::newsletters;
use crate::routes::{confirm, health_check, subscribe, home, login_form, login};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
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

    let server = run(tcp_listener, connection, email_client)?;

    Ok(Application { port, server })
}

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: Brevo,
) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
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
    })
    .listen(listener)?
    .run();

    Ok(server)
}
