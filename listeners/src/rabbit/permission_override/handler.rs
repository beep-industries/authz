use std::{convert::Infallible, sync::Arc};

use authz_core::domain::permission_override::{
    entities::{CreatePermissionOverrideInput, DeletePermissionOverrideInput, OverrideTarget},
    port::PermissionOverrideService,
};
use events_protobuf::communities_events::{
    DeletePermissionOverride, OverrideAction, UpsertPermissionOverride,
};
use tracing::{error, info, instrument, warn};

use crate::rabbit::consumers::AppState;

#[instrument(skip(state), fields(override_id = %input.override_id, channel_id = %input.channel_id))]
pub async fn upsert_permission_override(
    state: Arc<AppState>,
    input: UpsertPermissionOverride,
) -> Result<(), Infallible> {
    let permissions_bitmask = input
        .permission_bitmask
        .as_ref()
        .map(|p| p.value)
        .unwrap_or(0);

    let is_allow = input.action() == OverrideAction::Allow;

    info!(
        override_id = %input.override_id,
        channel_id = %input.channel_id,
        permissions_bitmask = %permissions_bitmask,
        is_allow = %is_allow,
        "Processing upsert permission override request"
    );

    // Extract target from oneof field
    let target = match &input.target {
        Some(override_target) => match &override_target.target {
            Some(events_protobuf::communities_events::upsert_permission_override::override_target::Target::UserId(user_id)) => {
                OverrideTarget::User(user_id.clone())
            }
            Some(events_protobuf::communities_events::upsert_permission_override::override_target::Target::RoleId(role_id)) => {
                OverrideTarget::Role(role_id.clone())
            }
            None => {
                warn!(
                    override_id = %input.override_id,
                    channel_id = %input.channel_id,
                    "No target specified in permission override, skipping"
                );
                return Ok(());
            }
        },
        None => {
            warn!(
                override_id = %input.override_id,
                channel_id = %input.channel_id,
                "No target specified in permission override, skipping"
            );
            return Ok(());
        }
    };

    match state
        .clone()
        .service
        .create(CreatePermissionOverrideInput {
            override_id: input.override_id.clone(),
            channel_id: input.channel_id.clone(),
            permission_bitmask: permissions_bitmask,
            is_allow,
            target,
        })
        .await
    {
        Ok(_) => {
            info!(
                override_id = %input.override_id,
                channel_id = %input.channel_id,
                "Successfully created/updated permission override"
            );
        }
        Err(e) => {
            error!(
                override_id = %input.override_id,
                channel_id = %input.channel_id,
                error = ?e,
                "Failed to create/update permission override"
            );
        }
    }
    Ok(())
}

#[instrument(skip(state), fields(override_id = %input.override_id))]
pub async fn delete_permission_override(
    state: Arc<AppState>,
    input: DeletePermissionOverride,
) -> Result<(), Infallible> {
    info!(
        override_id = %input.override_id,
        "Processing delete permission override request"
    );

    // Note: We need metadata to delete the override, but the protobuf event only provides override_id
    // The repository will lookup the metadata from the mapping or use a fallback approach
    // For now, we'll create a DeletePermissionOverrideInput with placeholder values
    // The repository handles the metadata lookup internally
    match state
        .clone()
        .service
        .delete(DeletePermissionOverrideInput {
            override_id: input.override_id.clone(),
            // These are placeholder values - the repository will look up actual metadata
            channel_id: String::new(),
            permission_bitmask: 0,
            is_allow: true,
            target: OverrideTarget::User(String::new()),
        })
        .await
    {
        Ok(_) => {
            info!(
                override_id = %input.override_id,
                "Successfully deleted permission override"
            );
        }
        Err(e) => {
            error!(
                override_id = %input.override_id,
                error = ?e,
                "Failed to delete permission override"
            );
        }
    }
    Ok(())
}
