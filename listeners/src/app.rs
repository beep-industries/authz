use authz_core::{application::create_repositories, domain::common::CoreError};
use tracing::{debug, info, instrument};

use crate::{
    config::Config,
    lapin::{RabbitClient, RabbitClientError},
    permissions_translations::BeepPermissions,
    rabbit::{
        channel::consumers::channel_consumers,
        consumers::{
            AppState,
            pool::{ConsumerPool, Consumers},
        },
        permission_override::consumers::permission_override_consumers,
        role::consumers::role_consumers,
        server::consumers::server_consumers,
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

        // Clone queue config to avoid borrow checker issues when moving other fields
        let queue_config = config.queue_config().clone();
        let rabbit_config = config.rabbit_config;
        let authzed_config = config.authzed_config;

        debug!("Connecting to RabbitMQ");
        let rabbit_client = RabbitClient::new(rabbit_config)
            .await
            .map_err(|e| AppError::RabbitError(e))?;
        info!("RabbitMQ client connected successfully");

        debug!("Creating permissions descriptor");
        let permissions_descriptor = BeepPermissions::new().descriptor();
        info!("Permissions descriptor created successfully");

        debug!("Creating authorization repositories");
        let authz_repositories = create_repositories(authzed_config, permissions_descriptor)
            .await
            .map_err(|e| AppError::RepositoriesCreationError(e))?;
        info!("Authorization repositories created successfully");

        let app_state = AppState::from(authz_repositories);

        debug!("Registering consumers");
        let server_consumers = server_consumers(&queue_config.server);
        let channel_consumers = channel_consumers(&queue_config.channel);
        let role_consumers = role_consumers(&queue_config.role);
        let permission_override_consumers =
            permission_override_consumers(&queue_config.permission_override);
        let consumers = Consumers::new()
            .merge(server_consumers)
            .merge(channel_consumers)
            .merge(role_consumers)
            .merge(permission_override_consumers);
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
