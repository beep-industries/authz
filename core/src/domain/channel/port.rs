use crate::domain::channel::{
    ChannelError,
    entities::{CreateChannelInput, DeleteChannelInput},
};

pub trait ChannelRepository: Send + Sync {
    fn create(&self, input: CreateChannelInput) -> impl Future<Output = Result<(), ChannelError>>;
    fn delete(&self, input: DeleteChannelInput) -> impl Future<Output = Result<(), ChannelError>>;
}

pub trait ChannelService: Send + Sync {
    fn create(&self, input: CreateChannelInput) -> impl Future<Output = Result<(), ChannelError>>;
    fn delete(&self, input: DeleteChannelInput) -> impl Future<Output = Result<(), ChannelError>>;
}
