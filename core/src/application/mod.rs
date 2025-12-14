use crate::{
    domain::common::{CoreError, service::Service},
    infrastructure::{
        authzed::{AuthZedClient, AuthZedConfig},
        server::repository::authzed::AuthzedServerRepository,
    },
};
use tracing::{info, instrument};

pub type AuthzService = Service<AuthzedServerRepository>;

pub struct AuthzRepositories {
    pub authzed_client: AuthZedClient,
    pub server_repository: AuthzedServerRepository,
}

#[instrument(skip_all)]
pub async fn create_repositories(
    authzed_config: AuthZedConfig,
) -> Result<AuthzRepositories, CoreError> {
    info!("Creating authorization repositories");

    let authzed_client = AuthZedClient::new(authzed_config)
        .await
        .map_err(|e| CoreError::StartupError { msg: e.to_string() })?;
    info!("AuthZed client created successfully");

    let server_repository = AuthzedServerRepository::new(authzed_client.clone());
    info!("Server repository initialized");

    let authz_repositories = AuthzRepositories {
        authzed_client,
        server_repository,
    };

    info!("All repositories created successfully");
    Ok(authz_repositories)
}

impl Into<AuthzService> for AuthzRepositories {
    fn into(self) -> AuthzService {
        AuthzService {
            server_repository: self.server_repository,
        }
    }
}
