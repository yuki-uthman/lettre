use letter::configuration::get_configuration;
use letter::email::Brevo;
use letter::startup::run;
use letter::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("letter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", config.application.port);
    let tcp_listener = TcpListener::bind(address).expect("Failed to bind port");
    let connection = PgPool::connect_lazy(config.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres.");

    let email_client = Brevo::with_secret(".secret");

    run(tcp_listener, connection, email_client)?.await
}
