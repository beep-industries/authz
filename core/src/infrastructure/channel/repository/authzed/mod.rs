use crate::{
    domain::channel::{
        ChannelError,
        entities::{CreateChannelInput, DeleteChannelInput},
        port::ChannelRepository,
    },
    infrastructure::authzed::AuthZedClient,
};
use tracing::{info, instrument};
pub mod entities;

#[derive(Clone)]
pub struct AuthzedChannelRepository {
    pub authzed_client: AuthZedClient,
}

impl AuthzedChannelRepository {
    pub fn new(authzed_client: AuthZedClient) -> Self {
        Self { authzed_client }
    }
}

impl ChannelRepository for AuthzedChannelRepository {
    #[instrument(skip(self), fields(channel_id = %input.channel_id, server_id = %input.server_id))]
    async fn create(&self, input: CreateChannelInput) -> Result<(), ChannelError> {
        info!(
            channel_id = %input.channel_id,
            server_id = %input.server_id,
            "Creating channel relationship in AuthZed"
        );

        let result = self
            .authzed_client
            .create_relationship(input)
            .await
            .map_err(|e| ChannelError::CreateChannelError { msg: e.to_string() });

        match &result {
            Ok(_) => info!("Channel relationship created successfully in AuthZed"),
            Err(e) => info!(error = ?e, "Failed to create channel relationship in AuthZed"),
        }

        result
    }

    #[instrument(skip(self), fields(channel_id = %input.channel_id))]
    async fn delete(&self, input: DeleteChannelInput) -> Result<(), ChannelError> {
        info!(
            channel_id = %input.channel_id,
            "Deleting channel relationships in AuthZed"
        );

        let result = self
            .authzed_client
            .filtered_delete(input)
            .await
            .map_err(|e| ChannelError::DeleteChannelError { msg: e.to_string() });

        match &result {
            Ok(_) => info!("Channel relationships deleted successfully in AuthZed"),
            Err(e) => info!(error = ?e, "Failed to delete channel relationships in AuthZed"),
        }

        result
    }
}
