pub mod server;
pub mod user;

pub type Id = String;

pub enum Entity {
    User,
    Server,
}

impl Into<String> for Entity {
    fn into(self) -> String {
        match self {
            Entity::User => "user".to_string(),
            Entity::Server => "sever".to_string(),
        }
    }
}

pub enum Relation {
    Owner,
}

impl Into<String> for Relation {
    fn into(self) -> String {
        match self {
            Relation::Owner => "owner".to_string(),
        }
    }
}