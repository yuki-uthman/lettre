use letter::configuration::get_configuration;
use letter::startup::build;
use letter::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let subscriber = get_subscriber("letter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration.");
    let app = build(config).await?;
    app.run().await?;

    Ok(())
}
