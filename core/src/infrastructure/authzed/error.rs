use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthzedError {
    #[error("Could not connect to spicedb: {msg}")]
    ConnectionError { msg: String },


    #[error("Could not put relationship: {msg}")]
    PutRelationshipError { msg: String },
}
