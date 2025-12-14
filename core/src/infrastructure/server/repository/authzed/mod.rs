use crate::{
    domain::server::{
        ServerError,
        entities::{CreateServerInput, DeleteServerInput},
        port::ServerRepository,
    },
    infrastructure::authzed::AuthZedClient,
};
use tracing::{info, instrument};
pub mod entities;

#[derive(Clone)]
pub struct AuthzedServerRepository {
    pub authzed_client: AuthZedClient,
}

impl AuthzedServerRepository {
    pub fn new(authzed_client: AuthZedClient) -> Self {
        Self { authzed_client }
    }
}

impl ServerRepository for AuthzedServerRepository {
    #[instrument(skip(self), fields(server_id = %input.server_id, owner_id = %input.owner_id))]
    async fn create(&self, input: CreateServerInput) -> Result<(), ServerError> {
        info!(
            server_id = %input.server_id,
            owner_id = %input.owner_id,
            "Creating server relationship in AuthZed"
        );

        let result = self
            .authzed_client
            .create_relationship(input)
            .await
            .map_err(|e| ServerError::CreateServerError { msg: e.to_string() });

        match &result {
            Ok(_) => info!("Server relationship created successfully in AuthZed"),
            Err(e) => info!(error = ?e, "Failed to create server relationship in AuthZed"),
        }

        result
    }

    #[instrument(skip(self), fields(server_id = %input.server_id))]
    async fn delete(&self, input: DeleteServerInput) -> Result<(), ServerError> {
        info!(
            server_id = %input.server_id,
            "Deleting server relationships in AuthZed"
        );

        let result = self
            .authzed_client
            .filtered_delete(input)
            .await
            .map_err(|e| ServerError::DeleteServerError { msg: e.to_string() });

        match &result {
            Ok(_) => info!("Server relationships deleted successfully in AuthZed"),
            Err(e) => info!(error = ?e, "Failed to delete server relationships in AuthZed"),
        }

        result
    }
}
