use tonic::transport::Channel;

use crate::{
    PermissionsServiceClient, SchemaServiceClient, WatchServiceClient,
    authzed::api::v1::{
        ObjectReference, Relationship, RelationshipUpdate, SubjectReference,
        WriteRelationshipsRequest, algebraic_subject_set::Operation,
    },
    infrastructure::authzed::error::AuthzedError,
};
pub mod entities;
pub mod error;

/// AuthZed client configuration
#[derive(Debug, Clone)]
pub struct AuthZedConfig {
    /// The endpoint URL (e.g., "grpc.authzed.com:443" or "localhost:50051")
    pub endpoint: String,
}

impl Default for AuthZedConfig {
    fn default() -> Self {
        Self {
            endpoint: "grpc.authzed.com:443".to_string(),
        }
    }
}

/// Main AuthZed client with all service clients
pub struct AuthZedClient {
    pub permissions: PermissionsServiceClient<Channel>,
    pub schema: SchemaServiceClient<Channel>,
    pub watch: WatchServiceClient<Channel>,
}

impl AuthZedClient {
    /// Create a new AuthZed client with the given configuration
    pub async fn new(config: AuthZedConfig) -> Result<Self, AuthzedError> {
        let channel = Self::create_channel(config).await?;

        Ok(Self {
            permissions: PermissionsServiceClient::new(channel.clone()),
            schema: SchemaServiceClient::new(channel.clone()),
            watch: WatchServiceClient::new(channel.clone()),
        })
    }

    async fn create_channel(config: AuthZedConfig) -> Result<Channel, AuthzedError> {
        let endpoint = Channel::from_shared(config.endpoint)
            .map_err(|e| AuthzedError::ConnectionError { msg: e.to_string() })?;

        let channel = endpoint
            .connect()
            .await
            .map_err(|e| AuthzedError::ConnectionError { msg: e.to_string() })?;

        Ok(channel)
    }

    pub async fn put_relationship(
        &mut self,
        operation: Operation,
        resource: impl Into<Option<ObjectReference>>,
        relationship: String,
        subject: impl Into<Option<SubjectReference>>,
    ) -> Result<(), AuthzedError> {
        let relationship = Relationship {
            resource: resource.into(),
            relation: relationship,
            subject: subject.into(),
            ..Default::default()
        };
        let update = RelationshipUpdate {
            operation: operation.into(),
            relationship: Some(relationship),
            ..Default::default()
        };
        let request = WriteRelationshipsRequest {
            updates: vec![update],
            ..Default::default()
        };

        self.permissions
            .write_relationships(request)
            .await
            .map_err(|e| AuthzedError::PutRelationshipError { msg: e.to_string() })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AuthZedConfig::default();
        assert_eq!(config.endpoint, "grpc.authzed.com:443");
    }
}
