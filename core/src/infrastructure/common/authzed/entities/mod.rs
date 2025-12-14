pub mod channel;
pub mod server;
pub mod user;

pub type Id = String;

pub enum Entity {
    User,
    Server,
    Channel,
}

impl Into<String> for Entity {
    fn into(self) -> String {
        match self {
            Entity::User => "user".to_string(),
            Entity::Server => "server".to_string(),
            Entity::Channel => "channel".to_string(),
        }
    }
}

pub enum Relation {
    Owner,
    Server,
}

impl Into<String> for Relation {
    fn into(self) -> String {
        match self {
            Relation::Owner => "owner".to_string(),
            Relation::Server => "server".to_string(),
        }
    }
}
