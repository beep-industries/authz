use std::{fmt::Display, sync::Arc};

use permission_translation::models::{CapabilityDescriptor, CapilityHexValue};

#[derive(Debug)]
pub enum Permissions {
    Administrator,    // Can do any action on any subject (channel, webhooks…) in a server.
    ManageServer,     // Can update a server (all CRUD except delete).
    ManageRoles,      // Can do all CRUD operations on all roles.
    CreateInvitation, // Can create server invites.
    ManageChannels,   // Can do all CRUD operations on every channel.
    ManageWebhooks,   // Can do all CRUD operations on every webhook.
    ViewChannels,     // Can see the channel and its contents (messages).
    SendMessages,     // Can send a message on the channel.
    ManageNicknames,  // Can update other users' nicknames.
    ChangeNickname,   // Can update your own nickname.
    ManageMessages,   // Can delete other users' messages.
    AttachFiles,      // Can upload images and files.
}

impl Into<CapilityHexValue> for Permissions {
    fn into(self) -> CapilityHexValue {
        match self {
            Permissions::Administrator => 0x1,
            Permissions::ManageServer => 0x2,
            Permissions::ManageRoles => 0x4,
            Permissions::CreateInvitation => 0x8,
            Permissions::ManageChannels => 0x10,
            Permissions::ManageWebhooks => 0x20,
            Permissions::ViewChannels => 0x40,
            Permissions::SendMessages => 0x80,
            Permissions::ManageNicknames => 0x100,
            Permissions::ChangeNickname => 0x200,
            Permissions::ManageMessages => 0x400,
            Permissions::AttachFiles => 0x800,
        }
    }
}

impl Display for Permissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permissions::Administrator => write!(f, "admin"),
            Permissions::ManageServer => write!(f, "manage"),
            Permissions::ManageRoles => write!(f, "manage_role"),
            Permissions::CreateInvitation => write!(f, "create_invitation"),
            Permissions::ManageChannels => write!(f, "manage_channels"),
            Permissions::ManageWebhooks => write!(f, "manage_webhooks"),
            Permissions::ViewChannels => write!(f, "view_channel"),
            Permissions::SendMessages => write!(f, "send_message"),
            Permissions::ManageNicknames => write!(f, "manage_nicknames"),
            Permissions::ChangeNickname => write!(f, "change_nickname"),
            Permissions::ManageMessages => write!(f, "manage_message"),
            Permissions::AttachFiles => write!(f, "attach_files"),
        }
    }
}

pub struct BeepPermissions(Arc<CapabilityDescriptor>);
impl BeepPermissions {
    pub fn new() -> Self {
        let mut descriptor = CapabilityDescriptor::new();
        Self::all_permissions().into_iter().for_each(|permission| {
            descriptor.insert(permission.to_string(), permission.into());
        });

        BeepPermissions(Arc::new(descriptor))
    }

    pub fn all_permissions() -> Vec<Permissions> {
        vec![
            Permissions::Administrator,
            Permissions::ManageServer,
            Permissions::ManageRoles,
            Permissions::CreateInvitation,
            Permissions::ManageChannels,
            Permissions::ManageWebhooks,
            Permissions::ViewChannels,
            Permissions::SendMessages,
            Permissions::ManageNicknames,
            Permissions::ChangeNickname,
            Permissions::ManageMessages,
            Permissions::AttachFiles,
        ]
    }

    pub fn descriptor(&self) -> Arc<CapabilityDescriptor> {
        Arc::clone(&self.0)
    }
}
