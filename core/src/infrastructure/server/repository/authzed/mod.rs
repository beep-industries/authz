use crate::{
    domain::server::{ServerError, entities::CreateServerInput, port::ServerRepository},
    infrastructure::authzed::AuthZedClient,
};
pub mod entities;

pub struct AuthzedServerRepository {
    pub authzed_client: AuthZedClient,
}

impl AuthzedServerRepository {
    pub fn new(authzed_client: AuthZedClient) -> Self {
        Self { authzed_client }
    }
}

impl ServerRepository for AuthzedServerRepository {
    async fn create(&self, input: CreateServerInput) -> Result<(), ServerError> {
        self.authzed_client
            .create_relationship(input)
            .await
            .map_err(|e| ServerError::CreateServerError { msg: e.to_string() })
    }
}
