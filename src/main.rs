use lettre::configuration::get_configuration;
use lettre::startup::run;
use lettre::telemetry::{get_subscriber, init_subscriber};
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("lettre".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", config.application_port);
    let tcp_listener = TcpListener::bind(address).expect("Failed to bind port");
    let connection = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    run(tcp_listener, connection)?.await
}
