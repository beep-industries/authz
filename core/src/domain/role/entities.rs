#[derive(Debug, Clone)]
pub struct CreateRoleInput {
    pub role_id: String,
    pub server_id: String,
    pub permissions_bitmask: u64,
}

#[derive(Debug, Clone)]
pub struct DeleteRoleInput {
    pub role_id: String,
}

#[derive(Debug, Clone)]
pub struct AssignMemberInput {
    pub user_id: String,
    pub role_id: String,
}

#[derive(Debug, Clone)]
pub struct RemoveMemberInput {
    pub user_id: String,
    pub role_id: String,
}
