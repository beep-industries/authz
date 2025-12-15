use permission_translation::models::CapabilityDescriptor;

/// Parse a permission bitmask and return the list of permission Display names
pub fn parse_permission_bitmask(bitmask: u64, descriptor: &CapabilityDescriptor) -> Vec<String> {
    let mut permissions = Vec::new();

    // Iterate through all permissions in the descriptor
    for (name, &hex_value) in descriptor.iter() {
        let hex_value_u64 = hex_value as u64;
        if bitmask & hex_value_u64 != 0 {
            permissions.push(name.clone());
        }
    }

    permissions
}

/// Convert a permission Display name to a server relation name
/// Returns None if the permission is not recognized
pub fn permission_display_to_server_relation(display_name: &str) -> Option<&'static str> {
    match display_name {
        "admin" => Some("administrator"),
        "manage" => Some("server_manager"),
        "manage_role" => Some("role_manager"),
        "create_invitation" => Some("invitation_creator"),
        "manage_channels" => Some("channel_manager"),
        "manage_webhooks" => Some("webhook_manager"),
        "view_channel" => Some("channel_viewer"),
        "send_message" => Some("message_sender"),
        "manage_nicknames" => Some("nickname_manager"),
        "change_nickname" => Some("nickname_changer"),
        "manage_message" => Some("message_manager"),
        "attach_files" => Some("file_attacher"),
        _ => None,
    }
}

/// Check if a permission is a channel-level permission
pub fn is_channel_permission(display_name: &str) -> bool {
    matches!(
        display_name,
        "send_message" | "view_channel" | "manage_message" | "attach_files" | "manage_webhooks"
    )
}

/// Convert a permission Display name to a channel relation name with grant/deny suffix
/// Returns None if the permission is not a valid channel permission
pub fn permission_display_to_channel_relation(
    display_name: &str,
    is_grant: bool,
) -> Option<String> {
    let suffix = if is_grant { "_grant" } else { "_deny" };

    let base_relation = match display_name {
        "send_message" => Some("send_message"),
        "view_channel" => Some("view_channel"),
        "manage_message" => Some("manage_message"),
        "attach_files" => Some("attach_files"),
        "manage_webhooks" => Some("manage_webhooks"),
        _ => None,
    };

    base_relation.map(|base| format!("{}{}", base, suffix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use permission_translation::models::CapabilityDescriptor;

    fn create_test_descriptor() -> CapabilityDescriptor {
        let mut descriptor = CapabilityDescriptor::new();
        descriptor.insert("admin".to_string(), 0x1);
        descriptor.insert("manage".to_string(), 0x2);
        descriptor.insert("manage_role".to_string(), 0x4);
        descriptor.insert("create_invitation".to_string(), 0x8);
        descriptor.insert("manage_channels".to_string(), 0x10);
        descriptor.insert("manage_webhooks".to_string(), 0x20);
        descriptor.insert("view_channel".to_string(), 0x40);
        descriptor.insert("send_message".to_string(), 0x80);
        descriptor.insert("manage_nicknames".to_string(), 0x100);
        descriptor.insert("change_nickname".to_string(), 0x200);
        descriptor.insert("manage_message".to_string(), 0x400);
        descriptor.insert("attach_files".to_string(), 0x800);
        descriptor
    }

    #[test]
    fn test_parse_permission_bitmask_single_bit() {
        let descriptor = create_test_descriptor();
        let permissions = parse_permission_bitmask(0x1, &descriptor);
        assert_eq!(permissions.len(), 1);
        assert!(permissions.contains(&"admin".to_string()));
    }

    #[test]
    fn test_parse_permission_bitmask_multiple_bits() {
        let descriptor = create_test_descriptor();
        let permissions = parse_permission_bitmask(0x88, &descriptor); // SendMessages | CreateInvitation
        assert_eq!(permissions.len(), 2);
        assert!(permissions.contains(&"send_message".to_string()));
        assert!(permissions.contains(&"create_invitation".to_string()));
    }

    #[test]
    fn test_parse_permission_bitmask_all_bits() {
        let descriptor = create_test_descriptor();
        let permissions = parse_permission_bitmask(0xFFF, &descriptor);
        assert_eq!(permissions.len(), 12);
    }

    #[test]
    fn test_permission_to_server_relation_mapping() {
        assert_eq!(
            permission_display_to_server_relation("admin"),
            Some("administrator")
        );
        assert_eq!(
            permission_display_to_server_relation("manage"),
            Some("server_manager")
        );
        assert_eq!(
            permission_display_to_server_relation("manage_role"),
            Some("role_manager")
        );
        assert_eq!(
            permission_display_to_server_relation("create_invitation"),
            Some("invitation_creator")
        );
        assert_eq!(
            permission_display_to_server_relation("manage_channels"),
            Some("channel_manager")
        );
        assert_eq!(
            permission_display_to_server_relation("manage_webhooks"),
            Some("webhook_manager")
        );
        assert_eq!(
            permission_display_to_server_relation("view_channel"),
            Some("channel_viewer")
        );
        assert_eq!(
            permission_display_to_server_relation("send_message"),
            Some("message_sender")
        );
        assert_eq!(
            permission_display_to_server_relation("manage_nicknames"),
            Some("nickname_manager")
        );
        assert_eq!(
            permission_display_to_server_relation("change_nickname"),
            Some("nickname_changer")
        );
        assert_eq!(
            permission_display_to_server_relation("manage_message"),
            Some("message_manager")
        );
        assert_eq!(
            permission_display_to_server_relation("attach_files"),
            Some("file_attacher")
        );
        assert_eq!(permission_display_to_server_relation("unknown"), None);
    }

    #[test]
    fn test_is_channel_permission() {
        assert!(is_channel_permission("send_message"));
        assert!(is_channel_permission("view_channel"));
        assert!(is_channel_permission("manage_message"));
        assert!(is_channel_permission("attach_files"));
        assert!(is_channel_permission("manage_webhooks"));

        assert!(!is_channel_permission("admin"));
        assert!(!is_channel_permission("manage"));
        assert!(!is_channel_permission("manage_role"));
        assert!(!is_channel_permission("create_invitation"));
    }

    #[test]
    fn test_permission_to_channel_relation_grant() {
        assert_eq!(
            permission_display_to_channel_relation("send_message", true),
            Some("send_message_grant".to_string())
        );
        assert_eq!(
            permission_display_to_channel_relation("view_channel", true),
            Some("view_channel_grant".to_string())
        );
        assert_eq!(
            permission_display_to_channel_relation("manage_message", true),
            Some("manage_message_grant".to_string())
        );
        assert_eq!(
            permission_display_to_channel_relation("attach_files", true),
            Some("attach_files_grant".to_string())
        );
        assert_eq!(
            permission_display_to_channel_relation("manage_webhooks", true),
            Some("manage_webhooks_grant".to_string())
        );
    }

    #[test]
    fn test_permission_to_channel_relation_deny() {
        assert_eq!(
            permission_display_to_channel_relation("send_message", false),
            Some("send_message_deny".to_string())
        );
        assert_eq!(
            permission_display_to_channel_relation("view_channel", false),
            Some("view_channel_deny".to_string())
        );
    }

    #[test]
    fn test_permission_to_channel_relation_invalid() {
        assert_eq!(permission_display_to_channel_relation("admin", true), None);
        assert_eq!(
            permission_display_to_channel_relation("manage_role", false),
            None
        );
    }
}
