use crate::{
    domain::common::{CoreError, service::Service},
    infrastructure::{
        authzed::{AuthZedClient, AuthZedConfig},
        channel::repository::authzed::AuthzedChannelRepository,
        permission_override::repository::authzed::AuthzedPermissionOverrideRepository,
        role::repository::authzed::AuthzedRoleRepository,
        server::repository::authzed::AuthzedServerRepository,
    },
};
use permission_translation::models::CapabilityDescriptor;
use std::sync::Arc;

pub type AuthzService = Service<
    AuthzedServerRepository,
    AuthzedChannelRepository,
    AuthzedRoleRepository,
    AuthzedPermissionOverrideRepository,
>;

pub struct AuthzRepositories {
    pub authzed_client: AuthZedClient,
    pub server_repository: AuthzedServerRepository,
    pub channel_repository: AuthzedChannelRepository,
    pub role_repository: AuthzedRoleRepository,
    pub permission_override_repository: AuthzedPermissionOverrideRepository,
}

pub async fn create_repositories(
    authzed_config: AuthZedConfig,
    permissions_descriptor: Arc<CapabilityDescriptor>,
) -> Result<AuthzRepositories, CoreError> {
    let authzed_client = AuthZedClient::new(authzed_config)
        .await
        .map_err(|e| CoreError::StartupError { msg: e.to_string() })?;
    let server_repository = AuthzedServerRepository::new(authzed_client.clone());
    let channel_repository = AuthzedChannelRepository::new(authzed_client.clone());
    let role_repository =
        AuthzedRoleRepository::new(authzed_client.clone(), permissions_descriptor.clone());
    let permission_override_repository = AuthzedPermissionOverrideRepository::new(
        authzed_client.clone(),
        permissions_descriptor.clone(),
    );
    let authz_repositories = AuthzRepositories {
        authzed_client,
        server_repository,
        channel_repository,
        role_repository,
        permission_override_repository,
    };
    Ok(authz_repositories)
}

impl Into<AuthzService> for AuthzRepositories {
    fn into(self) -> AuthzService {
        AuthzService {
            server_repository: self.server_repository,
            channel_repository: self.channel_repository,
            role_repository: self.role_repository,
            permission_override_repository: self.permission_override_repository,
        }
    }
}
