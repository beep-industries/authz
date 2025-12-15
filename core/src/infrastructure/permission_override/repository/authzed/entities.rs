use crate::{
    authzed::api::v1::{ObjectReference, Relationship, RelationshipUpdate, SubjectReference},
    domain::permission_override::entities::{CreatePermissionOverrideInput, OverrideTarget},
    infrastructure::{
        authzed::entities::Action,
        common::permissions::{
            is_channel_permission, parse_permission_bitmask, permission_display_to_channel_relation,
        },
    },
};
use permission_translation::models::CapabilityDescriptor;
use tracing::warn;

/// Convert CreatePermissionOverrideInput to a vector of Relationships (without action)
pub fn create_override_to_relationships(
    input: &CreatePermissionOverrideInput,
    descriptor: &CapabilityDescriptor,
) -> Vec<Relationship> {
    let mut relationships = Vec::new();

    // Parse permission bitmask to get permission names
    let permission_names = parse_permission_bitmask(input.permission_bitmask, descriptor);

    // Filter to only channel-level permissions
    let channel_permissions: Vec<_> = permission_names
        .into_iter()
        .filter(|perm| {
            let is_channel = is_channel_permission(perm);
            if !is_channel {
                warn!(
                    permission_name = %perm,
                    override_id = %input.override_id,
                    "Non-channel permission found in override bitmask, ignoring"
                );
            }
            is_channel
        })
        .collect();

    // Create relationship for each channel permission
    for permission_name in channel_permissions {
        if let Some(channel_relation) =
            permission_display_to_channel_relation(&permission_name, input.is_allow)
        {
            let subject_ref = match &input.target {
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

            let relationship = Relationship {
                resource: Some(ObjectReference {
                    object_type: "channel".to_string(),
                    object_id: input.channel_id.clone(),
                }),
                relation: channel_relation,
                subject: Some(subject_ref),
                optional_caveat: None,
                optional_expires_at: None,
            };

            relationships.push(relationship);
        }
    }

    relationships
}

/// Convert CreatePermissionOverrideInput to a vector of RelationshipUpdates
pub fn create_override_to_updates(
    input: &CreatePermissionOverrideInput,
    descriptor: &CapabilityDescriptor,
) -> Vec<RelationshipUpdate> {
    create_override_to_relationships(input, descriptor)
        .into_iter()
        .map(|rel| rel.create())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::permission_override::entities::OverrideTarget;
    use permission_translation::models::CapabilityDescriptor;

    fn create_test_descriptor() -> CapabilityDescriptor {
        let mut descriptor = CapabilityDescriptor::new();
        descriptor.insert("admin".to_string(), 0x1);
        descriptor.insert("manage".to_string(), 0x2);
        descriptor.insert("manage_webhooks".to_string(), 0x20);
        descriptor.insert("view_channel".to_string(), 0x40);
        descriptor.insert("send_message".to_string(), 0x80);
        descriptor.insert("manage_message".to_string(), 0x400);
        descriptor.insert("attach_files".to_string(), 0x800);
        descriptor
    }

    #[test]
    fn test_create_override_to_relationships_single_permission_user() {
        let descriptor = create_test_descriptor();
        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0x80, // send_message
            is_allow: true,
            target: OverrideTarget::User("user_789".to_string()),
        };

        let relationships = create_override_to_relationships(&input, &descriptor);

        assert_eq!(relationships.len(), 1);
        let rel = &relationships[0];
        assert_eq!(rel.resource.as_ref().unwrap().object_type, "channel");
        assert_eq!(rel.resource.as_ref().unwrap().object_id, "channel_456");
        assert_eq!(rel.relation, "send_message_grant");
        assert_eq!(
            rel.subject
                .as_ref()
                .unwrap()
                .object
                .as_ref()
                .unwrap()
                .object_type,
            "user"
        );
        assert_eq!(
            rel.subject
                .as_ref()
                .unwrap()
                .object
                .as_ref()
                .unwrap()
                .object_id,
            "user_789"
        );
    }

    #[test]
    fn test_create_override_to_relationships_single_permission_role() {
        let descriptor = create_test_descriptor();
        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0x40, // view_channel
            is_allow: false,          // deny
            target: OverrideTarget::Role("role_999".to_string()),
        };

        let relationships = create_override_to_relationships(&input, &descriptor);

        assert_eq!(relationships.len(), 1);
        let rel = &relationships[0];
        assert_eq!(rel.resource.as_ref().unwrap().object_type, "channel");
        assert_eq!(rel.resource.as_ref().unwrap().object_id, "channel_456");
        assert_eq!(rel.relation, "view_channel_deny");
        assert_eq!(
            rel.subject
                .as_ref()
                .unwrap()
                .object
                .as_ref()
                .unwrap()
                .object_type,
            "role"
        );
        assert_eq!(
            rel.subject
                .as_ref()
                .unwrap()
                .object
                .as_ref()
                .unwrap()
                .object_id,
            "role_999"
        );
        assert_eq!(rel.subject.as_ref().unwrap().optional_relation, "member");
    }

    #[test]
    fn test_create_override_to_relationships_multiple_permissions() {
        let descriptor = create_test_descriptor();
        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0xC0, // view_channel (0x40) | send_message (0x80)
            is_allow: true,
            target: OverrideTarget::User("user_789".to_string()),
        };

        let relationships = create_override_to_relationships(&input, &descriptor);

        // Should create 2 relationships
        assert_eq!(relationships.len(), 2);

        // Check that both relations exist
        let relations: Vec<_> = relationships.iter().map(|r| r.relation.as_str()).collect();
        assert!(relations.contains(&"view_channel_grant"));
        assert!(relations.contains(&"send_message_grant"));
    }

    #[test]
    fn test_create_override_filters_non_channel_permissions() {
        let descriptor = create_test_descriptor();
        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0x83, // admin (0x1) | manage (0x2) | send_message (0x80)
            is_allow: true,
            target: OverrideTarget::User("user_789".to_string()),
        };

        let relationships = create_override_to_relationships(&input, &descriptor);

        // Should only create 1 relationship for send_message (admin and manage are filtered)
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].relation, "send_message_grant");
    }

    #[test]
    fn test_create_override_to_relationships_all_channel_permissions() {
        let descriptor = create_test_descriptor();
        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0xCE0, // manage_webhooks | view_channel | send_message | manage_message | attach_files
            is_allow: false,
            target: OverrideTarget::User("user_789".to_string()),
        };

        let relationships = create_override_to_relationships(&input, &descriptor);

        // Should create 5 relationships (all channel permissions)
        assert_eq!(relationships.len(), 5);

        let relations: Vec<_> = relationships.iter().map(|r| r.relation.as_str()).collect();
        assert!(relations.contains(&"manage_webhooks_deny"));
        assert!(relations.contains(&"view_channel_deny"));
        assert!(relations.contains(&"send_message_deny"));
        assert!(relations.contains(&"manage_message_deny"));
        assert!(relations.contains(&"attach_files_deny"));
    }

    #[test]
    fn test_create_override_to_updates() {
        let descriptor = create_test_descriptor();
        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0x80,
            is_allow: true,
            target: OverrideTarget::User("user_789".to_string()),
        };

        let updates = create_override_to_updates(&input, &descriptor);

        assert_eq!(updates.len(), 1);
        // Verify it's a RelationshipUpdate with CREATE operation
        assert_eq!(updates[0].operation, 1); // CREATE operation
    }
}
