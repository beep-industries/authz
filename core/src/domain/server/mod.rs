use thiserror::Error;

pub mod entities;
pub mod port;
pub mod service;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Create server error: {msg}")]
    CreateServerError { msg: String },
}

