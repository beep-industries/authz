use crate::{
    authzed::api::v1::{ObjectReference, Relationship, RelationshipUpdate, SubjectReference},
    domain::permission_override::entities::CreatePermissionOverrideInput,
    infrastructure::{
        authzed::entities::Action,
        common::permissions::{
            is_channel_permission, parse_permission_bitmask, permission_display_to_channel_relation,
        },
    },
};
use permission_translation::models::CapabilityDescriptor;
use tracing::warn;

/// Create channel permission relationships pointing to permission_override object
/// These relationships look like: channel:X#send_message_grant@permission_override:Y#granted_to
pub fn create_channel_override_relationships(
    input: &CreatePermissionOverrideInput,
    descriptor: &CapabilityDescriptor,
) -> Vec<RelationshipUpdate> {
    let mut updates = Vec::new();

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

    // Determine the relation suffix on permission_override (granted_to or denied_to)
    let override_relation = if input.is_allow {
        "granted_to"
    } else {
        "denied_to"
    };

    // Create relationship for each channel permission
    for permission_name in channel_permissions {
        if let Some(channel_relation) =
            permission_display_to_channel_relation(&permission_name, input.is_allow)
        {
            // Create: channel:X#send_message_grant@permission_override:Y#granted_to
            let relationship = Relationship {
                resource: Some(ObjectReference {
                    object_type: "channel".to_string(),
                    object_id: input.channel_id.clone(),
                }),
                relation: channel_relation,
                subject: Some(SubjectReference {
                    object: Some(ObjectReference {
                        object_type: "permission_override".to_string(),
                        object_id: input.override_id.clone(),
                    }),
                    optional_relation: override_relation.to_string(),
                }),
                optional_caveat: None,
                optional_expires_at: None,
            };

            updates.push(relationship.create());
        }
    }

    updates
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
    fn test_create_channel_override_relationships_single_permission() {
        let descriptor = create_test_descriptor();
        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0x80, // send_message
            is_allow: true,
            target: OverrideTarget::User("user_789".to_string()),
        };

        let updates = create_channel_override_relationships(&input, &descriptor);

        assert_eq!(updates.len(), 1);
        // Verify it's a RelationshipUpdate with CREATE operation
        assert_eq!(updates[0].operation, 1); // CREATE operation

        let rel = updates[0].relationship.as_ref().unwrap();
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
            "permission_override"
        );
        assert_eq!(
            rel.subject
                .as_ref()
                .unwrap()
                .object
                .as_ref()
                .unwrap()
                .object_id,
            "override_123"
        );
        assert_eq!(
            rel.subject.as_ref().unwrap().optional_relation,
            "granted_to"
        );
    }

    #[test]
    fn test_create_channel_override_relationships_deny() {
        let descriptor = create_test_descriptor();
        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0x40, // view_channel
            is_allow: false,          // deny
            target: OverrideTarget::Role("role_999".to_string()),
        };

        let updates = create_channel_override_relationships(&input, &descriptor);

        assert_eq!(updates.len(), 1);
        let rel = updates[0].relationship.as_ref().unwrap();
        assert_eq!(rel.relation, "view_channel_deny");
        assert_eq!(
            rel.subject
                .as_ref()
                .unwrap()
                .object
                .as_ref()
                .unwrap()
                .object_type,
            "permission_override"
        );
        assert_eq!(rel.subject.as_ref().unwrap().optional_relation, "denied_to");
    }

    #[test]
    fn test_create_channel_override_relationships_multiple_permissions() {
        let descriptor = create_test_descriptor();
        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0xC0, // view_channel (0x40) | send_message (0x80)
            is_allow: true,
            target: OverrideTarget::User("user_789".to_string()),
        };

        let updates = create_channel_override_relationships(&input, &descriptor);

        // Should create 2 relationships
        assert_eq!(updates.len(), 2);
    }

    #[test]
    fn test_create_channel_override_filters_non_channel_permissions() {
        let descriptor = create_test_descriptor();
        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0x83, // admin (0x1) | manage (0x2) | send_message (0x80)
            is_allow: true,
            target: OverrideTarget::User("user_789".to_string()),
        };

        let updates = create_channel_override_relationships(&input, &descriptor);

        // Should only create 1 relationship for send_message (admin and manage are filtered)
        assert_eq!(updates.len(), 1);
    }
}
