use crate::{
    config::ServerQueues,
    rabbit::{
        consumers::{AppState, pool::Consumers},
        server::handler::{create_server, delete_server},
    },
};

pub fn server_consumers(queue_config: &ServerQueues) -> Consumers<AppState> {
    Consumers::new()
        .add(&queue_config.create_server, create_server)
        .add(&queue_config.delete_server, delete_server)
}
