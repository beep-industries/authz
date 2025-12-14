use clap::Parser;
use listeners::{app::App, config::Config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting authorization service");

    let config = Config::parse();
    tracing::debug!(?config, "Parsed configuration");

    tracing::info!(
        "Loading queue configuration from {:?}",
        config.queue_config_path
    );
    let config = config.with_queue_config()?;
    tracing::info!("Queue configuration loaded successfully");

    let app = App::new(config).await?;
    tracing::info!("Application initialized successfully");

    app.start().await;

    Ok(())
}
