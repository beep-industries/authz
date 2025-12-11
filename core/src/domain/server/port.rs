use crate::domain::server::{ServerError, entities::CreateServerInput};

pub trait ServerRepository: Send + Sync {
    fn create(&self, input: CreateServerInput) -> impl Future<Output = Result<(), ServerError>>;
}

pub trait ServerService: Send + Sync {
    fn create(&self, input: CreateServerInput) -> impl Future<Output = Result<(), ServerError>>;
}
