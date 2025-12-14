use crate::{
    domain::common::{CoreError, service::Service},
    infrastructure::{
        authzed::{AuthZedClient, AuthZedConfig},
        channel::repository::authzed::AuthzedChannelRepository,
        server::repository::authzed::AuthzedServerRepository,
    },
};

pub type AuthzService = Service<AuthzedServerRepository, AuthzedChannelRepository>;

pub struct AuthzRepositories {
    pub authzed_client: AuthZedClient,
    pub server_repository: AuthzedServerRepository,
    pub channel_repository: AuthzedChannelRepository,
}

pub async fn create_repositories(
    authzed_config: AuthZedConfig,
) -> Result<AuthzRepositories, CoreError> {
    let authzed_client = AuthZedClient::new(authzed_config)
        .await
        .map_err(|e| CoreError::StartupError { msg: e.to_string() })?;
    let server_repository = AuthzedServerRepository::new(authzed_client.clone());
    let channel_repository = AuthzedChannelRepository::new(authzed_client.clone());
    let authz_repositories = AuthzRepositories {
        authzed_client,
        server_repository,
        channel_repository,
    };
    Ok(authz_repositories)
}

impl Into<AuthzService> for AuthzRepositories {
    fn into(self) -> AuthzService {
        AuthzService {
            server_repository: self.server_repository,
            channel_repository: self.channel_repository,
        }
    }
}
