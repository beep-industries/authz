use std::{convert::Infallible, sync::Arc};

use authz_core::domain::role::{
    entities::{AssignMemberInput, CreateRoleInput, DeleteRoleInput, RemoveMemberInput},
    port::RoleService,
};
use events_protobuf::communities_events::{
    DeleteRole, MemberAssignedToRole, MemberRemovedFromRole, UpsertRole,
};
use tracing::{error, info, instrument};

use crate::rabbit::consumers::AppState;

#[instrument(skip(state), fields(role_id = %input.role_id, server_id = %input.server_id, permissions_bitmask = %input.permissions_bitmask.as_ref().map(|p| p.value).unwrap_or(0)))]
pub async fn upsert_role(state: Arc<AppState>, input: UpsertRole) -> Result<(), Infallible> {
    let permissions_bitmask = input
        .permissions_bitmask
        .as_ref()
        .map(|p| p.value)
        .unwrap_or(0);

    info!(
        role_id = %input.role_id,
        server_id = %input.server_id,
        permissions_bitmask = %permissions_bitmask,
        "Processing upsert role request"
    );

    match state
        .clone()
        .service
        .create(CreateRoleInput {
            role_id: input.role_id.clone(),
            server_id: input.server_id.clone(),
            permissions_bitmask,
        })
        .await
    {
        Ok(_) => {
            info!(
                role_id = %input.role_id,
                server_id = %input.server_id,
                "Successfully created/updated role"
            );
        }
        Err(e) => {
            error!(
                role_id = %input.role_id,
                server_id = %input.server_id,
                error = ?e,
                "Failed to create/update role"
            );
        }
    }
    Ok(())
}

#[instrument(skip(state), fields(role_id = %input.role_id))]
pub async fn delete_role(state: Arc<AppState>, input: DeleteRole) -> Result<(), Infallible> {
    info!(
        role_id = %input.role_id,
        "Processing delete role request"
    );

    match state
        .clone()
        .service
        .delete(DeleteRoleInput {
            role_id: input.role_id.clone(),
        })
        .await
    {
        Ok(_) => {
            info!(
                role_id = %input.role_id,
                "Successfully deleted role"
            );
        }
        Err(e) => {
            error!(
                role_id = %input.role_id,
                error = ?e,
                "Failed to delete role"
            );
        }
    }
    Ok(())
}

#[instrument(skip(state), fields(user_id = %input.user_id, role_id = %input.role_id))]
pub async fn assign_member_to_role(
    state: Arc<AppState>,
    input: MemberAssignedToRole,
) -> Result<(), Infallible> {
    info!(
        user_id = %input.user_id,
        role_id = %input.role_id,
        "Processing assign member to role request"
    );

    match state
        .clone()
        .service
        .assign_member(AssignMemberInput {
            user_id: input.user_id.clone(),
            role_id: input.role_id.clone(),
        })
        .await
    {
        Ok(_) => {
            info!(
                user_id = %input.user_id,
                role_id = %input.role_id,
                "Successfully assigned member to role"
            );
        }
        Err(e) => {
            error!(
                user_id = %input.user_id,
                role_id = %input.role_id,
                error = ?e,
                "Failed to assign member to role"
            );
        }
    }
    Ok(())
}

#[instrument(skip(state), fields(user_id = %input.user_id, role_id = %input.role_id))]
pub async fn remove_member_from_role(
    state: Arc<AppState>,
    input: MemberRemovedFromRole,
) -> Result<(), Infallible> {
    info!(
        user_id = %input.user_id,
        role_id = %input.role_id,
        "Processing remove member from role request"
    );

    match state
        .clone()
        .service
        .remove_member(RemoveMemberInput {
            user_id: input.user_id.clone(),
            role_id: input.role_id.clone(),
        })
        .await
    {
        Ok(_) => {
            info!(
                user_id = %input.user_id,
                role_id = %input.role_id,
                "Successfully removed member from role"
            );
        }
        Err(e) => {
            error!(
                user_id = %input.user_id,
                role_id = %input.role_id,
                error = ?e,
                "Failed to remove member from role"
            );
        }
    }
    Ok(())
}
