use crate::domain::server::{
    ServerError,
    entities::{CreateServerInput, DeleteServerInput},
};

pub trait ServerRepository: Send + Sync {
    fn create(&self, input: CreateServerInput) -> impl Future<Output = Result<(), ServerError>>;
    fn delete(&self, input: DeleteServerInput) -> impl Future<Output = Result<(), ServerError>>;
}

pub trait ServerService: Send + Sync {
    fn create(&self, input: CreateServerInput) -> impl Future<Output = Result<(), ServerError>>;
    fn delete(&self, input: DeleteServerInput) -> impl Future<Output = Result<(), ServerError>>;
}
