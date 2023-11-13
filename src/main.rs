use lettre::configuration::get_configuration;
use lettre::startup::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", config.application_port);
    let tcp_listener = TcpListener::bind(address).expect("Failed to bind port");
    run(tcp_listener)?.await
}
