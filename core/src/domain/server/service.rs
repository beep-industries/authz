use crate::domain::{
    common::service::Service,
    server::{
        ServerError,
        entities::CreateServerInput,
        port::{ServerRepository, ServerService},
    },
};
use tracing::{info, instrument};

impl<S> ServerService for Service<S>
where
    S: ServerRepository,
{
    #[instrument(skip(self), fields(server_id = %input.server_id, owner_id = %input.owner_id))]
    async fn create(&self, input: CreateServerInput) -> Result<(), ServerError> {
        info!(
            server_id = %input.server_id,
            owner_id = %input.owner_id,
            "Creating server in domain service"
        );
        let result = self.server_repository.create(input).await;
        match &result {
            Ok(_) => info!("Server created successfully in domain service"),
            Err(e) => info!(error = ?e, "Failed to create server in domain service"),
        }
        result
    }
}
