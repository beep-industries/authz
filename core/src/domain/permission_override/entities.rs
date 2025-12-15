#[derive(Debug, Clone)]
pub enum OverrideTarget {
    User(String),
    Role(String),
}

#[derive(Debug, Clone)]
pub struct CreatePermissionOverrideInput {
    pub override_id: String,
    pub channel_id: String,
    pub permission_bitmask: u64,
    pub is_allow: bool, // true = grant, false = deny
    pub target: OverrideTarget,
}

#[derive(Debug, Clone)]
pub struct DeletePermissionOverrideInput {
    pub override_id: String,
    // Store metadata for deletion since we need to reconstruct relationships
    pub channel_id: String,
    pub permission_bitmask: u64,
    pub is_allow: bool,
    pub target: OverrideTarget,
}
