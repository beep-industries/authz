use crate::{
    authzed::api::v1::{Relationship, RelationshipFilter},
    domain::channel::entities::{CreateChannelInput, DeleteChannelInput},
    infrastructure::common::authzed::entities::{Relation, channel::Channel, server::Server},
};

impl Into<(Channel, Server)> for CreateChannelInput {
    fn into(self) -> (Channel, Server) {
        let channel = Channel::from(self.channel_id);
        let server = Server::from(self.server_id);
        (channel, server)
    }
}

impl From<CreateChannelInput> for Relationship {
    fn from(input: CreateChannelInput) -> Self {
        let (channel, server): (Channel, Server) = input.into();
        Relationship {
            resource: Some(channel.into()),
            relation: Relation::Server.into(),
            subject: Some(server.into()),
            ..Default::default()
        }
    }
}

impl From<DeleteChannelInput> for RelationshipFilter {
    fn from(input: DeleteChannelInput) -> Self {
        RelationshipFilter {
            resource_type: "channel".to_string(),
            optional_resource_id: input.channel_id,
            optional_resource_id_prefix: String::new(),
            optional_relation: String::new(),
            optional_subject_filter: None,
        }
    }
}
