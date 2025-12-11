use crate::{
    authzed::api::v1::Relationship,
    domain::server::entities::CreateServerInput,
    infrastructure::common::authzed::entities::{Relation, server::Server, user::User},
};

impl Into<(User, Server)> for CreateServerInput {
    fn into(self) -> (User, Server) {
        let user = User::from(self.owner_id);
        let server = Server::from(self.server_id);
        (user, server)
    }
}

impl From<CreateServerInput> for Relationship {
    fn from(input: CreateServerInput) -> Self {
        let (user, server): (User, Server) = input.into();
        Relationship {
            resource: Some(server.into()),
            relation: Relation::Owner.into(),
            subject: Some(user.into()),
            ..Default::default()
        }
    }
}

