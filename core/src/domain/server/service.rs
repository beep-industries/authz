use crate::domain::{
    common::service::Service,
    server::{
        ServerError,
        entities::CreateServerInput,
        port::{ServerRepository, ServerService},
    },
};

impl<S> ServerService for Service<S>
where
    S: ServerRepository,
{
    async fn create(&self, input: CreateServerInput) -> Result<(), ServerError> {
        self.server_repository.create(input).await
    }
}
