use crate::{
    domain::permission_override::{
        PermissionOverrideError,
        entities::{CreatePermissionOverrideInput, DeletePermissionOverrideInput, OverrideTarget},
        port::PermissionOverrideRepository,
    },
    infrastructure::authzed::AuthZedClient,
};
use permission_translation::models::CapabilityDescriptor;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};

pub mod entities;

/// Metadata stored for each permission override to enable deletion
#[derive(Debug, Clone)]
pub struct OverrideMetadata {
    pub channel_id: String,
    pub permission_bitmask: u64,
    pub is_allow: bool,
    pub target: OverrideTarget,
}

#[derive(Clone)]
pub struct AuthzedPermissionOverrideRepository {
    pub authzed_client: AuthZedClient,
    pub permissions_descriptor: Arc<CapabilityDescriptor>,
    pub override_mapping: Arc<RwLock<HashMap<String, OverrideMetadata>>>,
}

impl AuthzedPermissionOverrideRepository {
    pub fn new(
        authzed_client: AuthZedClient,
        permissions_descriptor: Arc<CapabilityDescriptor>,
    ) -> Self {
        Self {
            authzed_client,
            permissions_descriptor,
            override_mapping: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl PermissionOverrideRepository for AuthzedPermissionOverrideRepository {
    #[instrument(skip(self), fields(override_id = %input.override_id, channel_id = %input.channel_id))]
    async fn create(
        &self,
        input: CreatePermissionOverrideInput,
    ) -> Result<(), PermissionOverrideError> {
        info!(
            override_id = %input.override_id,
            channel_id = %input.channel_id,
            permission_bitmask = %input.permission_bitmask,
            is_allow = %input.is_allow,
            "Creating permission override relationships in AuthZed"
        );

        // Convert input to relationship updates
        let updates = entities::create_override_to_updates(&input, &self.permissions_descriptor);

        if updates.is_empty() {
            warn!(
                override_id = %input.override_id,
                channel_id = %input.channel_id,
                permission_bitmask = %input.permission_bitmask,
                "No valid channel permissions found in bitmask, no relationships created"
            );
            // Still store the metadata for consistency
        } else {
            // Write all relationships in bulk
            self.authzed_client
                .write_relationships(updates)
                .await
                .map_err(|e| PermissionOverrideError::CreateOverrideError { msg: e.to_string() })?;
        }

        // Store metadata for future deletion
        let metadata = OverrideMetadata {
            channel_id: input.channel_id.clone(),
            permission_bitmask: input.permission_bitmask,
            is_allow: input.is_allow,
            target: input.target.clone(),
        };
        self.override_mapping
            .write()
            .await
            .insert(input.override_id.clone(), metadata);

        info!("Permission override relationships created successfully in AuthZed");
        Ok(())
    }

    #[instrument(skip(self), fields(override_id = %input.override_id))]
    async fn delete(
        &self,
        input: DeletePermissionOverrideInput,
    ) -> Result<(), PermissionOverrideError> {
        info!(
            override_id = %input.override_id,
            "Deleting permission override relationships in AuthZed"
        );

        // Look up metadata from mapping
        let metadata_opt = self
            .override_mapping
            .read()
            .await
            .get(&input.override_id)
            .cloned();

        let metadata = match metadata_opt {
            Some(m) => m,
            None => {
                // If metadata not found, try using the input data directly
                warn!(
                    override_id = %input.override_id,
                    "Override metadata not found in mapping, using input data for deletion"
                );
                OverrideMetadata {
                    channel_id: input.channel_id.clone(),
                    permission_bitmask: input.permission_bitmask,
                    is_allow: input.is_allow,
                    target: input.target.clone(),
                }
            }
        };

        // Reconstruct the relationships to delete
        let delete_input = CreatePermissionOverrideInput {
            override_id: input.override_id.clone(),
            channel_id: metadata.channel_id,
            permission_bitmask: metadata.permission_bitmask,
            is_allow: metadata.is_allow,
            target: metadata.target,
        };

        let relationships =
            entities::create_override_to_relationships(&delete_input, &self.permissions_descriptor);

        // Delete each relationship
        for relationship in relationships {
            self.authzed_client
                .delete_relationship(relationship)
                .await
                .map_err(|e| PermissionOverrideError::DeleteOverrideError { msg: e.to_string() })?;
        }

        // Remove from mapping
        self.override_mapping
            .write()
            .await
            .remove(&input.override_id);

        info!("Permission override relationships deleted successfully in AuthZed");
        Ok(())
    }
}
