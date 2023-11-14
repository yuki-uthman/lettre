use env_logger::Env;
use lettre::configuration::get_configuration;
use lettre::startup::run;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Make info the default log level
    // to change the log level, use the RUST_LOG environment variable
    // ie. RUST_LOG=debug cargo run
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", config.application_port);
    let tcp_listener = TcpListener::bind(address).expect("Failed to bind port");
    let connection = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    run(tcp_listener, connection)?.await
}
