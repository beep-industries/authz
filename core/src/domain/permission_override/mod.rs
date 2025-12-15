use thiserror::Error;

pub mod entities;
pub mod port;
pub mod service;

#[derive(Debug, Error)]
pub enum PermissionOverrideError {
    #[error("Create override error: {msg}")]
    CreateOverrideError { msg: String },
    #[error("Delete override error: {msg}")]
    DeleteOverrideError { msg: String },
}
