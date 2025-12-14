use std::{convert::Infallible, sync::Arc};

use authz_core::domain::server::{entities::CreateServerInput, port::ServerService};
use events_protobuf::communities_events::CreateServer;

use crate::rabbit::consumers::AppState;

pub async fn create_server(state: Arc<AppState>, input: CreateServer) -> Result<(), Infallible> {
    let _ = state
        .clone()
        .service
        .create(CreateServerInput {
            owner_id: input.owner_id,
            server_id: input.server_id,
        })
        .await;
    Ok(())
}
