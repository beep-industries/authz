use authz_core::{application::create_repositories, domain::common::CoreError};
use tracing::{debug, info, instrument};

use crate::{
    config::Config,
    lapin::{RabbitClient, RabbitClientError},
    rabbit::{
        consumers::{
            AppState,
            pool::{ConsumerPool, Consumers},
        },
        server::consumers::create_server_consumers,
    },
};

pub struct App {
    app_state: AppState,
    consumer_pool: ConsumerPool<AppState>,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("RabbitMQ error: {0}")]
    RabbitError(RabbitClientError),
    // Other error variants can be added here
    #[error("Repositories creation error: {0}")]
    RepositoriesCreationError(CoreError),
}

impl App {
    #[instrument(skip_all)]
    pub async fn new(config: Config) -> Result<Self, AppError> {
        info!("Initializing application");

        debug!("Connecting to RabbitMQ");
        let rabbit_client = RabbitClient::new(config.rabbit_config)
            .await
            .map_err(|e| AppError::RabbitError(e))?;
        info!("RabbitMQ client connected successfully");

        debug!("Creating authorization repositories");
        let authz_repositories = create_repositories(config.authzed_config)
            .await
            .map_err(|e| AppError::RepositoriesCreationError(e))?;
        info!("Authorization repositories created successfully");

        let app_state = AppState::from(authz_repositories);

        debug!("Registering consumers");
        let server_consumers = create_server_consumers();
        let consumers = Consumers::new().merge(server_consumers);
        let consumer_count = consumers.count();
        info!(consumer_count, "Registered consumers");

        let consumer_pool: ConsumerPool<AppState> =
            ConsumerPool::new(rabbit_client, app_state.clone(), consumers);

        info!("Application initialization complete");
        Ok(Self {
            app_state,
            consumer_pool,
        })
    }

    #[instrument(skip_all)]
    pub async fn start(self) {
        info!("Starting consumer pool");
        self.consumer_pool.start().await;
    }
}
