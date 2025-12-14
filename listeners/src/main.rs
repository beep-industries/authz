use clap::Parser;
use listeners::{app::App, config::Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let config = Config::parse();
    let app = App::new(config).await?;
    app.start().await;
    Ok(())
}
