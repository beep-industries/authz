use crate::domain::{channel::port::ChannelRepository, server::port::ServerRepository};

#[derive(Clone)]
pub struct Service<S, C>
where
    S: ServerRepository,
    C: ChannelRepository,
{
    pub(crate) server_repository: S,
    pub(crate) channel_repository: C,
}

impl<S, C> Service<S, C>
where
    S: ServerRepository,
    C: ChannelRepository,
{
    pub fn new(server_repository: S, channel_repository: C) -> Self {
        Self {
            server_repository,
            channel_repository,
        }
    }
}
