use std::{convert::Infallible, sync::Arc};

use authz_core::domain::server::{entities::CreateServerInput, port::ServerService};
use events_protobuf::communities_events::CreateServer;

use crate::rabbit::consumers::AppState;

pub async fn create_server(state: Arc<AppState>, input: CreateServer) -> Result<(), Infallible> {
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
            eprintln!(
                "✅ Successfully created server: {} owned by {}",
                input.server_id, input.owner_id
            );
        }
        Err(e) => {
            eprintln!(
                "❌ Failed to create server: {} owned by {}: {:?}",
                input.server_id, input.owner_id, e
            );
        }
    }
    Ok(())
}
