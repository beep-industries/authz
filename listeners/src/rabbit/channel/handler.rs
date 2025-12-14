use std::{convert::Infallible, sync::Arc};

use authz_core::domain::channel::{
    entities::{CreateChannelInput, DeleteChannelInput},
    port::ChannelService,
};
use events_protobuf::communities_events::{ChannelCreated, ChannelDeleted};
use tracing::{error, info, instrument};

use crate::rabbit::consumers::AppState;

#[instrument(skip(state), fields(channel_id = %input.channel_id, server_id = %input.server_id))]
pub async fn create_channel(state: Arc<AppState>, input: ChannelCreated) -> Result<(), Infallible> {
    info!(
        channel_id = %input.channel_id,
        server_id = %input.server_id,
        "Processing create channel request"
    );

    match state
        .clone()
        .service
        .create(CreateChannelInput {
            channel_id: input.channel_id.clone(),
            server_id: input.server_id.clone(),
        })
        .await
    {
        Ok(_) => {
            info!(
                channel_id = %input.channel_id,
                server_id = %input.server_id,
                "Successfully created channel"
            );
        }
        Err(e) => {
            error!(
                channel_id = %input.channel_id,
                server_id = %input.server_id,
                error = ?e,
                "Failed to create channel"
            );
        }
    }
    Ok(())
}

#[instrument(skip(state), fields(channel_id = %input.channel_id))]
pub async fn delete_channel(state: Arc<AppState>, input: ChannelDeleted) -> Result<(), Infallible> {
    info!(
        channel_id = %input.channel_id,
        "Processing delete channel request"
    );

    match state
        .clone()
        .service
        .delete(DeleteChannelInput {
            channel_id: input.channel_id.clone(),
        })
        .await
    {
        Ok(_) => {
            info!(
                channel_id = %input.channel_id,
                "Successfully deleted channel"
            );
        }
        Err(e) => {
            error!(
                channel_id = %input.channel_id,
                error = ?e,
                "Failed to delete channel"
            );
        }
    }
    Ok(())
}
