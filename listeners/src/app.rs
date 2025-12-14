use authz_core::{application::create_repositories, domain::common::CoreError};

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
    pub async fn new(config: Config) -> Result<Self, AppError> {
        let authz_repositories = create_repositories(config.authzed_config)
            .await
            .map_err(|e| AppError::RepositoriesCreationError(e))?;
        let app_state = AppState::from(authz_repositories);
        // let app_state = AppState::from(config.authz_repositories);
        let rabbit_client = RabbitClient::new(config.rabbit_config)
            .await
            .map_err(|e| AppError::RabbitError(e))?;
        let server_consumers = create_server_consumers();
        let consumers = Consumers::new().merge(server_consumers);
        let consumer_pool: ConsumerPool<AppState> =
            ConsumerPool::new(rabbit_client, app_state.clone(), consumers);
        Ok(Self {
            app_state,
            consumer_pool,
        })
    }

    pub async fn start(self) {
        self.consumer_pool.start().await;
    }
}
