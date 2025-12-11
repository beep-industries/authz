use crate::{
    domain::common::{CoreError, service::Service},
    infrastructure::{
        authzed::{AuthZedClient, AuthZedConfig},
        server::repository::authzed::AuthzedServerRepository,
    },
};

pub type AuthzService = Service<AuthzedServerRepository>;

pub struct AuthzRepositories {
    pub authzed_client: AuthZedClient,
    pub server_repository: AuthzedServerRepository,
}

pub async fn create_repositories(
    authzed_config: AuthZedConfig,
) -> Result<AuthzRepositories, CoreError> {
    let authzed_client = AuthZedClient::new(authzed_config)
        .await
        .map_err(|e| CoreError::StartupError { msg: e.to_string() })?;
    let server_repository = AuthzedServerRepository::new(authzed_client.clone()); 
    let authz_repositories = AuthzRepositories {
        authzed_client,
        server_repository,
    };
    Ok(authz_repositories)
}

impl Into<AuthzService> for AuthzRepositories {
    fn into(self) -> AuthzService {
        AuthzService {
            server_repository: self.server_repository,
        }
    }
}
