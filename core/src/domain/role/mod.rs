use thiserror::Error;

pub mod entities;
pub mod port;
pub mod service;

#[derive(Debug, Error)]
pub enum RoleError {
    #[error("Create role error: {msg}")]
    CreateRoleError { msg: String },
    #[error("Delete role error: {msg}")]
    DeleteRoleError { msg: String },
    #[error("Assign member error: {msg}")]
    AssignMemberError { msg: String },
    #[error("Remove member error: {msg}")]
    RemoveMemberError { msg: String },
}
