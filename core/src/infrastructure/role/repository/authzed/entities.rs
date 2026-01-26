use crate::{
    authzed::api::v1::{
        ObjectReference, Relationship, RelationshipFilter, RelationshipUpdate, SubjectReference,
    },
    domain::role::entities::{
        AssignMemberInput, CreateRoleInput, DeleteRoleInput, RemoveMemberInput,
    },
    infrastructure::{
        authzed::entities::Action,
        common::permissions::{parse_permission_bitmask, permission_display_to_server_relation},
    },
};
use permission_translation::models::CapabilityDescriptor;
use tracing::warn;

/// Create the base role->server relationship
pub fn create_role_server_relationship(input: &CreateRoleInput) -> Relationship {
    Relationship {
        resource: Some(ObjectReference {
            object_type: "role".to_string(),
            object_id: input.role_id.clone(),
        }),
        relation: "server".to_string(),
        subject: Some(SubjectReference {
            object: Some(ObjectReference {
                object_type: "server".to_string(),
                object_id: input.server_id.clone(),
            }),
            optional_relation: String::new(),
        }),
        optional_caveat: None,
        optional_expires_at: None,
    }
}

/// Convert CreateRoleInput to a vector of RelationshipUpdates
/// Creates role->server relationship and server->role#member relationships for each permission
pub fn create_role_to_updates(
    input: &CreateRoleInput,
    descriptor: &CapabilityDescriptor,
) -> Vec<RelationshipUpdate> {
    let mut updates = Vec::new();

    // Create base role->server relationship
    let role_server_relationship = create_role_server_relationship(input);
    updates.push(role_server_relationship.touch());

    // Parse permission bitmask to get permission names
    let permission_names = parse_permission_bitmask(input.permissions_bitmask, descriptor);

    // For each permission, create server#relation@role#member relationship
    for permission_name in permission_names {
        if let Some(server_relation) = permission_display_to_server_relation(&permission_name) {
            let relationship = Relationship {
                resource: Some(ObjectReference {
                    object_type: "server".to_string(),
                    object_id: input.server_id.clone(),
                }),
                relation: server_relation.to_string(),
                subject: Some(SubjectReference {
                    object: Some(ObjectReference {
                        object_type: "role".to_string(),
                        object_id: input.role_id.clone(),
                    }),
                    optional_relation: "member".to_string(),
                }),
                optional_caveat: None,
                optional_expires_at: None,
            };
            updates.push(relationship.touch());
        } else {
            warn!(
                permission_name = %permission_name,
                "Unknown permission name, skipping"
            );
        }
    }

    updates
}

/// Create a RelationshipFilter for deleting all relationships where role is the resource
pub fn create_role_resource_filter(input: &DeleteRoleInput) -> RelationshipFilter {
    RelationshipFilter {
        resource_type: "role".to_string(),
        optional_resource_id: input.role_id.clone(),
        optional_relation: String::new(),
        optional_subject_filter: None,
        optional_resource_id_prefix: String::new(),
    }
}

/// Create a RelationshipFilter for deleting all server relationships where role is the subject
pub fn create_role_subject_filter(input: &DeleteRoleInput) -> RelationshipFilter {
    RelationshipFilter {
        resource_type: "server".to_string(),
        optional_resource_id: String::new(),
        optional_relation: String::new(),
        optional_subject_filter: Some(crate::authzed::api::v1::SubjectFilter {
            subject_type: "role".to_string(),
            optional_subject_id: input.role_id.clone(),
            optional_relation: Some(crate::authzed::api::v1::subject_filter::RelationFilter {
                relation: "member".to_string(),
            }),
        }),
        optional_resource_id_prefix: String::new(),
    }
}

/// Convert AssignMemberInput to Relationship
pub fn assign_member_to_relationship(input: &AssignMemberInput) -> Relationship {
    Relationship {
        resource: Some(ObjectReference {
            object_type: "role".to_string(),
            object_id: input.role_id.clone(),
        }),
        relation: "member".to_string(),
        subject: Some(SubjectReference {
            object: Some(ObjectReference {
                object_type: "user".to_string(),
                object_id: input.user_id.clone(),
            }),
            optional_relation: String::new(),
        }),
        optional_caveat: None,
        optional_expires_at: None,
    }
}

/// Convert RemoveMemberInput to Relationship (for deletion)
pub fn remove_member_to_relationship(input: &RemoveMemberInput) -> Relationship {
    Relationship {
        resource: Some(ObjectReference {
            object_type: "role".to_string(),
            object_id: input.role_id.clone(),
        }),
        relation: "member".to_string(),
        subject: Some(SubjectReference {
            object: Some(ObjectReference {
                object_type: "user".to_string(),
                object_id: input.user_id.clone(),
            }),
            optional_relation: String::new(),
        }),
        optional_caveat: None,
        optional_expires_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use permission_translation::models::CapabilityDescriptor;

    fn create_test_descriptor() -> CapabilityDescriptor {
        let mut descriptor = CapabilityDescriptor::new();
        descriptor.insert("admin".to_string(), 0x1);
        descriptor.insert("manage".to_string(), 0x2);
        descriptor.insert("send_message".to_string(), 0x80);
        descriptor.insert("create_invitation".to_string(), 0x8);
        descriptor
    }

    #[test]
    fn test_create_role_server_relationship() {
        let input = CreateRoleInput {
            role_id: "role_123".to_string(),
            server_id: "server_456".to_string(),
            permissions_bitmask: 0x1,
        };

        let relationship = create_role_server_relationship(&input);

        assert_eq!(relationship.resource.as_ref().unwrap().object_type, "role");
        assert_eq!(
            relationship.resource.as_ref().unwrap().object_id,
            "role_123"
        );
        assert_eq!(relationship.relation, "server");
        assert_eq!(
            relationship
                .subject
                .as_ref()
                .unwrap()
                .object
                .as_ref()
                .unwrap()
                .object_type,
            "server"
        );
        assert_eq!(
            relationship
                .subject
                .as_ref()
                .unwrap()
                .object
                .as_ref()
                .unwrap()
                .object_id,
            "server_456"
        );
    }

    #[test]
    fn test_create_role_to_updates_single_permission() {
        let descriptor = create_test_descriptor();
        let input = CreateRoleInput {
            role_id: "role_123".to_string(),
            server_id: "server_456".to_string(),
            permissions_bitmask: 0x1, // admin
        };

        let updates = create_role_to_updates(&input, &descriptor);

        // Should have 2 updates: role->server and server->role#member
        assert_eq!(updates.len(), 2);
    }

    #[test]
    fn test_create_role_to_updates_multiple_permissions() {
        let descriptor = create_test_descriptor();
        let input = CreateRoleInput {
            role_id: "role_123".to_string(),
            server_id: "server_456".to_string(),
            permissions_bitmask: 0x88, // send_message (0x80) | create_invitation (0x8)
        };

        let updates = create_role_to_updates(&input, &descriptor);

        // Should have 3 updates: role->server + 2 permission relationships
        assert_eq!(updates.len(), 3);
    }

    #[test]
    fn test_create_role_resource_filter() {
        let input = DeleteRoleInput {
            role_id: "role_123".to_string(),
        };

        let filter = create_role_resource_filter(&input);

        assert_eq!(filter.resource_type, "role");
        assert_eq!(filter.optional_resource_id, "role_123");
    }

    #[test]
    fn test_create_role_subject_filter() {
        let input = DeleteRoleInput {
            role_id: "role_123".to_string(),
        };

        let filter = create_role_subject_filter(&input);

        assert_eq!(filter.resource_type, "server");
        assert!(filter.optional_subject_filter.is_some());
        let subject_filter = filter.optional_subject_filter.unwrap();
        assert_eq!(subject_filter.subject_type, "role");
        assert_eq!(subject_filter.optional_subject_id, "role_123");
    }

    #[test]
    fn test_assign_member_to_relationship() {
        let input = AssignMemberInput {
            user_id: "user_789".to_string(),
            role_id: "role_123".to_string(),
        };

        let relationship = assign_member_to_relationship(&input);

        assert_eq!(relationship.resource.as_ref().unwrap().object_type, "role");
        assert_eq!(
            relationship.resource.as_ref().unwrap().object_id,
            "role_123"
        );
        assert_eq!(relationship.relation, "member");
        assert_eq!(
            relationship
                .subject
                .as_ref()
                .unwrap()
                .object
                .as_ref()
                .unwrap()
                .object_type,
            "user"
        );
        assert_eq!(
            relationship
                .subject
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
    fn test_remove_member_to_relationship() {
        let input = RemoveMemberInput {
            user_id: "user_789".to_string(),
            role_id: "role_123".to_string(),
        };

        let relationship = remove_member_to_relationship(&input);

        assert_eq!(relationship.resource.as_ref().unwrap().object_type, "role");
        assert_eq!(
            relationship.resource.as_ref().unwrap().object_id,
            "role_123"
        );
        assert_eq!(relationship.relation, "member");
        assert_eq!(
            relationship
                .subject
                .as_ref()
                .unwrap()
                .object
                .as_ref()
                .unwrap()
                .object_type,
            "user"
        );
        assert_eq!(
            relationship
                .subject
                .as_ref()
                .unwrap()
                .object
                .as_ref()
                .unwrap()
                .object_id,
            "user_789"
        );
    }
}
