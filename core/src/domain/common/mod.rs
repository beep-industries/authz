use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Service could not start: {msg}")]
    StartupError { msg: String },
}
