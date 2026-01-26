use crate::{
    domain::role::{
        RoleError,
        entities::{AssignMemberInput, CreateRoleInput, DeleteRoleInput, RemoveMemberInput},
        port::RoleRepository,
    },
    infrastructure::authzed::AuthZedClient,
};
use permission_translation::models::CapabilityDescriptor;
use std::sync::Arc;
use tracing::{info, instrument, warn};

pub mod entities;

#[derive(Clone)]
pub struct AuthzedRoleRepository {
    pub authzed_client: AuthZedClient,
    pub permissions_descriptor: Arc<CapabilityDescriptor>,
}

impl AuthzedRoleRepository {
    pub fn new(
        authzed_client: AuthZedClient,
        permissions_descriptor: Arc<CapabilityDescriptor>,
    ) -> Self {
        Self {
            authzed_client,
            permissions_descriptor,
        }
    }
}

impl RoleRepository for AuthzedRoleRepository {
    #[instrument(skip(self), fields(role_id = %input.role_id, server_id = %input.server_id, permissions_bitmask = %input.permissions_bitmask))]
    async fn create(&self, input: CreateRoleInput) -> Result<(), RoleError> {
        info!(
            role_id = %input.role_id,
            server_id = %input.server_id,
            permissions_bitmask = %input.permissions_bitmask,
            "Creating/updating role relationships in AuthZed"
        );

        // First, delete all existing server permission relations for this role (for update semantics)
        // This ensures we replace permissions instead of merging them
        info!(
            role_id = %input.role_id,
            "Deleting existing server permission relationships for role"
        );
        let subject_filter = entities::create_role_subject_filter(&DeleteRoleInput {
            role_id: input.role_id.clone(),
        });
        self.authzed_client
            .filtered_delete(subject_filter)
            .await
            .map_err(|e| RoleError::CreateRoleError { msg: e.to_string() })?;

        // Convert input to relationship updates
        let updates = entities::create_role_to_updates(&input, &self.permissions_descriptor);

        if updates.is_empty() {
            warn!(
                role_id = %input.role_id,
                "No valid relationships to create for role"
            );
            // Still create the base role->server relationship
            let base_relationship = entities::create_role_server_relationship(&input);
            self.authzed_client
                .touch_relationship(base_relationship)
                .await
                .map_err(|e| RoleError::CreateRoleError { msg: e.to_string() })?;
        } else {
            // Write all relationships in bulk
            self.authzed_client
                .write_relationships(updates)
                .await
                .map_err(|e| RoleError::CreateRoleError { msg: e.to_string() })?;
        }

        info!("Role relationships created/updated successfully in AuthZed");
        Ok(())
    }

    #[instrument(skip(self), fields(role_id = %input.role_id))]
    async fn delete(&self, input: DeleteRoleInput) -> Result<(), RoleError> {
        info!(
            role_id = %input.role_id,
            "Deleting role relationships in AuthZed"
        );

        // Delete all relationships where role is the resource
        let resource_filter = entities::create_role_resource_filter(&input);
        self.authzed_client
            .filtered_delete(resource_filter)
            .await
            .map_err(|e| RoleError::DeleteRoleError { msg: e.to_string() })?;

        // Delete all server permission relations where this role is the subject
        let subject_filter = entities::create_role_subject_filter(&input);
        self.authzed_client
            .filtered_delete(subject_filter)
            .await
            .map_err(|e| RoleError::DeleteRoleError { msg: e.to_string() })?;

        info!("Role relationships deleted successfully in AuthZed");
        Ok(())
    }

    #[instrument(skip(self), fields(user_id = %input.user_id, role_id = %input.role_id))]
    async fn assign_member(&self, input: AssignMemberInput) -> Result<(), RoleError> {
        info!(
            user_id = %input.user_id,
            role_id = %input.role_id,
            "Assigning member to role in AuthZed"
        );

        let relationship = entities::assign_member_to_relationship(&input);
        self.authzed_client
            .create_relationship(relationship)
            .await
            .map_err(|e| RoleError::AssignMemberError { msg: e.to_string() })?;

        info!("Member assigned to role successfully in AuthZed");
        Ok(())
    }

    #[instrument(skip(self), fields(user_id = %input.user_id, role_id = %input.role_id))]
    async fn remove_member(&self, input: RemoveMemberInput) -> Result<(), RoleError> {
        info!(
            user_id = %input.user_id,
            role_id = %input.role_id,
            "Removing member from role in AuthZed"
        );

        let relationship = entities::remove_member_to_relationship(&input);
        self.authzed_client
            .delete_relationship(relationship)
            .await
            .map_err(|e| RoleError::RemoveMemberError { msg: e.to_string() })?;

        info!("Member removed from role successfully in AuthZed");
        Ok(())
    }
}
