use lettre::startup::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let tcp_listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind port 8000");
    run(tcp_listener)?.await
}
