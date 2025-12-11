use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthzedError {
    #[error("Could not connect to spicedb: {msg}")]
    ConnectionError { msg: String },

    #[error("Could not write relationship: {msg}")]
    WriteRelationshipError { msg: String },

    #[error("Could not write relationships: {msg}")]
    WriteRelationshipsError { msg: String },

    #[error("Could not delete relationship: {msg}")]
    DeleteRelationshipError { msg: String },
}
