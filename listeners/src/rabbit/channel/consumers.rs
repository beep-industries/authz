use crate::{
    config::ChannelQueues,
    rabbit::{
        consumers::{AppState, pool::Consumers},
        channel::handler::{create_channel, delete_channel},
    },
};

pub fn channel_consumers(queue_config: &ChannelQueues) -> Consumers<AppState> {
    Consumers::new()
        .add(&queue_config.create_channel, create_channel)
        .add(&queue_config.delete_channel, delete_channel)
}
