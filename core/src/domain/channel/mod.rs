use thiserror::Error;

pub mod entities;
pub mod port;
pub mod service;

#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("Create channel error: {msg}")]
    CreateChannelError { msg: String },
    #[error("Delete channel error: {msg}")]
    DeleteChannelError { msg: String },
}
