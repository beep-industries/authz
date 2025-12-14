#[derive(Debug, Clone)]
pub struct CreateServerInput {
    pub owner_id: String,
    pub server_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteServerInput {
    pub server_id: String,
}
