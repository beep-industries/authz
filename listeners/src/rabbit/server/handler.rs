use std::{convert::Infallible, sync::Arc};

use authz_core::domain::server::{entities::CreateServerInput, port::ServerService};
use events_protobuf::communities_events::CreateServer;
use tracing::{error, info, instrument};

use crate::rabbit::consumers::AppState;

#[instrument(skip(state), fields(server_id = %input.server_id, owner_id = %input.owner_id))]
pub async fn create_server(state: Arc<AppState>, input: CreateServer) -> Result<(), Infallible> {
    info!(
        server_id = %input.server_id,
        owner_id = %input.owner_id,
        "Processing create server request"
    );

    match state
        .clone()
        .service
        .create(CreateServerInput {
            owner_id: input.owner_id.clone(),
            server_id: input.server_id.clone(),
        })
        .await
    {
        Ok(_) => {
            info!(
                server_id = %input.server_id,
                owner_id = %input.owner_id,
                "Successfully created server"
            );
        }
        Err(e) => {
            error!(
                server_id = %input.server_id,
                owner_id = %input.owner_id,
                error = ?e,
                "Failed to create server"
            );
        }
    }
    Ok(())
}
