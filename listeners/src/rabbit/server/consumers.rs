use crate::{
    config::ServerQueues,
    rabbit::{
        consumers::{AppState, pool::Consumers},
        server::handler::create_server,
    },
};

pub fn create_server_consumers(queue_config: &ServerQueues) -> Consumers<AppState> {
    Consumers::new().add(&queue_config.create_server, create_server)
}
