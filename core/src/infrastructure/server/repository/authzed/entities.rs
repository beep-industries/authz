use crate::{
    authzed::api::v1::{Relationship, RelationshipFilter},
    domain::server::entities::{CreateServerInput, DeleteServerInput},
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

impl From<DeleteServerInput> for RelationshipFilter {
    fn from(input: DeleteServerInput) -> Self {
        RelationshipFilter {
            resource_type: "server".to_string(),
            optional_resource_id: input.server_id,
            optional_resource_id_prefix: String::new(),
            optional_relation: String::new(),
            optional_subject_filter: None,
        }
    }
}
