use crate::domain::{
    channel::port::ChannelRepository, permission_override::port::PermissionOverrideRepository,
    role::port::RoleRepository, server::port::ServerRepository,
};

#[derive(Clone)]
pub struct Service<S, C, R, P>
where
    S: ServerRepository,
    C: ChannelRepository,
    R: RoleRepository,
    P: PermissionOverrideRepository,
{
    pub(crate) server_repository: S,
    pub(crate) channel_repository: C,
    pub(crate) role_repository: R,
    pub(crate) permission_override_repository: P,
}

impl<S, C, R, P> Service<S, C, R, P>
where
    S: ServerRepository,
    C: ChannelRepository,
    R: RoleRepository,
    P: PermissionOverrideRepository,
{
    pub fn new(
        server_repository: S,
        channel_repository: C,
        role_repository: R,
        permission_override_repository: P,
    ) -> Self {
        Self {
            server_repository,
            channel_repository,
            role_repository,
            permission_override_repository,
        }
    }
}
