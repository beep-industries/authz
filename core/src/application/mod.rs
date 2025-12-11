use crate::{
    domain::{common::CoreError, service::Service},
    infrastructure::authzed::{AuthZedClient, AuthZedConfig},
};

pub type AuthzService = Service;

pub struct AuthzRepositories {
    pub authz_client: AuthZedClient,
}

pub async fn create_repositories(
    authzed_config: AuthZedConfig,
) -> Result<AuthzRepositories, CoreError> {
    let authz_client = AuthZedClient::new(authzed_config)
        .await
        .map_err(|e| CoreError::StartupError { msg: e.to_string() })?;
    let authz_repositories = AuthzRepositories { authz_client };
    Ok(authz_repositories)
}

impl Into<AuthzService> for AuthzRepositories {
    fn into(self) -> AuthzService {
        AuthzService {}
    }
}
