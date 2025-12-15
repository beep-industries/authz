use crate::{
    authzed::api::v1::{ObjectReference, Relationship, RelationshipFilter, SubjectReference},
    domain::permission_override::{
        PermissionOverrideError,
        entities::{CreatePermissionOverrideInput, DeletePermissionOverrideInput, OverrideTarget},
        port::PermissionOverrideRepository,
    },
    infrastructure::authzed::AuthZedClient,
};
use permission_translation::models::CapabilityDescriptor;
use std::sync::Arc;
use tracing::{info, instrument, warn};

pub mod entities;

#[derive(Clone)]
pub struct AuthzedPermissionOverrideRepository {
    pub authzed_client: AuthZedClient,
    pub permissions_descriptor: Arc<CapabilityDescriptor>,
}

impl AuthzedPermissionOverrideRepository {
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

        // Create permission_override object with channel relationship
        let override_channel_rel = Relationship {
            resource: Some(ObjectReference {
                object_type: "permission_override".to_string(),
                object_id: input.override_id.clone(),
            }),
            relation: "channel".to_string(),
            subject: Some(SubjectReference {
                object: Some(ObjectReference {
                    object_type: "channel".to_string(),
                    object_id: input.channel_id.clone(),
                }),
                optional_relation: String::new(),
            }),
            optional_caveat: None,
            optional_expires_at: None,
        };

        self.authzed_client
            .create_relationship(override_channel_rel)
            .await
            .map_err(|e| PermissionOverrideError::CreateOverrideError { msg: e.to_string() })?;

        // Store target in granted_to or denied_to based on is_allow
        let target_relation_name = if input.is_allow {
            "granted_to"
        } else {
            "denied_to"
        };

        let target_subject = match &input.target {
            OverrideTarget::User(user_id) => SubjectReference {
                object: Some(ObjectReference {
                    object_type: "user".to_string(),
                    object_id: user_id.clone(),
                }),
                optional_relation: String::new(),
            },
            OverrideTarget::Role(role_id) => SubjectReference {
                object: Some(ObjectReference {
                    object_type: "role".to_string(),
                    object_id: role_id.clone(),
                }),
                optional_relation: "member".to_string(),
            },
        };

        let target_relation = Relationship {
            resource: Some(ObjectReference {
                object_type: "permission_override".to_string(),
                object_id: input.override_id.clone(),
            }),
            relation: target_relation_name.to_string(),
            subject: Some(target_subject),
            optional_caveat: None,
            optional_expires_at: None,
        };

        self.authzed_client
            .create_relationship(target_relation)
            .await
            .map_err(|e| PermissionOverrideError::CreateOverrideError { msg: e.to_string() })?;

        // Create channel permission relationships pointing to permission_override
        // channel:X#send_message_grant@permission_override:Y#granted_to
        let channel_updates =
            entities::create_channel_override_relationships(&input, &self.permissions_descriptor);

        if channel_updates.is_empty() {
            warn!(
                override_id = %input.override_id,
                channel_id = %input.channel_id,
                permission_bitmask = %input.permission_bitmask,
                "No valid channel permissions found in bitmask, no permission relationships created"
            );
        } else {
            // Write all channel permission relationships in bulk
            self.authzed_client
                .write_relationships(channel_updates)
                .await
                .map_err(|e| PermissionOverrideError::CreateOverrideError { msg: e.to_string() })?;
        }

        info!("Permission override object and relationships created successfully in AuthZed");
        Ok(())
    }

    #[instrument(skip(self), fields(override_id = %input.override_id))]
    async fn delete(
        &self,
        input: DeletePermissionOverrideInput,
    ) -> Result<(), PermissionOverrideError> {
        info!(
            override_id = %input.override_id,
            "Deleting permission override object and relationships in AuthZed"
        );

        // Delete all relationships where permission_override is the resource
        // This deletes:
        // - permission_override#channel
        // - permission_override#granted_to or permission_override#denied_to
        let override_resource_filter = RelationshipFilter {
            resource_type: "permission_override".to_string(),
            optional_resource_id: input.override_id.clone(),
            optional_relation: String::new(),
            optional_subject_filter: None,
            optional_resource_id_prefix: String::new(),
        };

        self.authzed_client
            .filtered_delete(override_resource_filter)
            .await
            .map_err(|e| PermissionOverrideError::DeleteOverrideError { msg: e.to_string() })?;

        // Delete all channel relationships where permission_override is the subject
        // This deletes: channel:X#*_grant@permission_override:Y#granted_to
        //           and: channel:X#*_deny@permission_override:Y#denied_to
        let override_subject_filter = RelationshipFilter {
            resource_type: "channel".to_string(),
            optional_resource_id: String::new(),
            optional_relation: String::new(),
            optional_subject_filter: Some(crate::authzed::api::v1::SubjectFilter {
                subject_type: "permission_override".to_string(),
                optional_subject_id: input.override_id.clone(),
                optional_relation: None,
            }),
            optional_resource_id_prefix: String::new(),
        };

        self.authzed_client
            .filtered_delete(override_subject_filter)
            .await
            .map_err(|e| PermissionOverrideError::DeleteOverrideError { msg: e.to_string() })?;

        info!("Permission override object and all relationships deleted successfully in AuthZed");
        Ok(())
    }
}
