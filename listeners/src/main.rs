use authz_core::{application::create_repositories, infrastructure::authzed::AuthZedConfig};
use clap::Parser;
use listeners::{config::Config, rabbit::consumers::AppState};

#[tokio::main]
async fn main() {
    let config = Config::parse();
    // let authzed_config = AuthZedConfig {
    //     endpoint: "test".to_string(),
    // };
    // let authz_repositories = create_repositories(authzed_config).await.unwrap();
    // let app_state = AppState::from(authz_repositories);
}
