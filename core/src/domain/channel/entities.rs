#[derive(Debug, Clone)]
pub struct CreateChannelInput {
    pub channel_id: String,
    pub server_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteChannelInput {
    pub channel_id: String,
}
