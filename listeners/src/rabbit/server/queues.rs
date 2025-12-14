use crate::rabbit::{
    consumers::{AppState, pool::Consumers},
    server::handler::create_server,
};

pub fn create_handler() -> Consumers<AppState> {
    Consumers::new().add("test", create_server)
}
