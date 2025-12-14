use crate::rabbit::{
    consumers::{AppState, pool::Consumers},
    server::handler::create_server,
};

pub fn create_server_consumers() -> Consumers<AppState> {
    Consumers::new().add("test", create_server)
}
