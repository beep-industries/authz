use std::sync::Arc;

use clap::Parser;
use tokio::sync::RwLock;
use tonic::transport::Channel;

use crate::{
    PermissionsServiceClient,
    authzed::api::v1::{
        DeleteRelationshipsRequest, ReadRelationshipsRequest, Relationship, RelationshipFilter,
        RelationshipUpdate, WriteRelationshipsRequest,
    },
    infrastructure::authzed::{entities::Action, error::AuthzedError},
};
use futures::StreamExt;
use tonic::service::Interceptor;
use tracing::{debug, error, info, instrument};

pub mod entities;
pub mod error;

/// AuthZed client configuration
#[derive(Debug, Clone, Parser)]
pub struct AuthZedConfig {
    /// The endpoint URL (e.g., "grpc.authzed.com:443" or "localhost:50051")
    #[arg(
        long = "authzed-endpoint",
        env = "AUTHZED_ENDPOINT",
        default_value = "localhost:50051"
    )]
    pub endpoint: String,

    /// The preshared key for authentication
    #[arg(long = "authzed-token", env = "AUTHZED_TOKEN")]
    pub token: Option<String>,
}

/// Main AuthZed client with all service clients
#[derive(Clone)]
pub struct AuthZedClient {
    permissions: Arc<
        RwLock<
            PermissionsServiceClient<
                tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>,
            >,
        >,
    >,
    // schema: SchemaServiceClient<Channel>,
}

impl AuthZedClient {
    async fn permissions(
        &self,
    ) -> tokio::sync::RwLockWriteGuard<
        '_,
        PermissionsServiceClient<
            tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>,
        >,
    > {
        self.permissions.write().await
    }

    /// Create a new AuthZed client with the given configuration
    #[instrument(skip_all, fields(endpoint = %config.endpoint))]
    pub async fn new(config: AuthZedConfig) -> Result<Self, AuthzedError> {
        info!(endpoint = %config.endpoint, "Creating AuthZed client");

        let channel = Self::create_channel(&config).await?;

        // Always use an interceptor, even if token is empty
        let token = config.token.unwrap_or_default();
        let has_token = !token.is_empty();
        debug!(has_token, "Configuring authentication interceptor");

        let interceptor = AuthInterceptor { token };
        let permissions = Arc::new(RwLock::new(PermissionsServiceClient::with_interceptor(
            channel.clone(),
            interceptor,
        )));

        info!("AuthZed client created successfully");
        Ok(Self {
            permissions,
            // schema: SchemaServiceClient::new(channel.clone()),
        })
    }

    #[instrument(skip_all, fields(endpoint = %config.endpoint))]
    async fn create_channel(config: &AuthZedConfig) -> Result<Channel, AuthzedError> {
        // Add http:// scheme if not present
        let endpoint_url =
            if config.endpoint.starts_with("http://") || config.endpoint.starts_with("https://") {
                config.endpoint.clone()
            } else {
                format!("http://{}", config.endpoint)
            };

        debug!(endpoint_url = %endpoint_url, "Creating gRPC channel");

        let endpoint = Channel::from_shared(endpoint_url.clone()).map_err(|e| {
            error!(endpoint_url = %endpoint_url, error = %e, "Invalid endpoint URL");
            AuthzedError::ConnectionError { msg: e.to_string() }
        })?;

        info!(endpoint_url = %endpoint_url, "Connecting to AuthZed");
        let channel = endpoint.connect().await.map_err(|e| {
            error!(endpoint_url = %endpoint_url, error = %e, "Failed to connect to AuthZed");
            AuthzedError::ConnectionError { msg: e.to_string() }
        })?;

        info!("Successfully connected to AuthZed");
        Ok(channel)
    }

    #[instrument(skip_all)]
    pub async fn create_relationship(
        &self,
        relationship: impl Into<Relationship>,
    ) -> Result<(), AuthzedError> {
        let relationship: Relationship = relationship.into();
        debug!(
            resource_type = relationship.resource.as_ref().map(|r| r.object_type.as_str()),
            resource_id = relationship.resource.as_ref().map(|r| r.object_id.as_str()),
            relation = %relationship.relation,
            "Creating relationship"
        );
        self.write_relationship(relationship.create()).await?;
        info!("Relationship created successfully");
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn delete_relationship(
        &self,
        relationship: impl Into<Relationship>,
    ) -> Result<(), AuthzedError> {
        let relationship: Relationship = relationship.into();
        debug!(
            resource_type = relationship.resource.as_ref().map(|r| r.object_type.as_str()),
            resource_id = relationship.resource.as_ref().map(|r| r.object_id.as_str()),
            relation = %relationship.relation,
            "Deleting relationship"
        );
        self.write_relationship(relationship.delete()).await?;
        info!("Relationship deleted successfully");
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn touch_relationship(
        &self,
        relationship: impl Into<Relationship>,
    ) -> Result<(), AuthzedError> {
        let relationship: Relationship = relationship.into();
        debug!(
            resource_type = relationship.resource.as_ref().map(|r| r.object_type.as_str()),
            resource_id = relationship.resource.as_ref().map(|r| r.object_id.as_str()),
            relation = %relationship.relation,
            "Touching relationship"
        );
        self.write_relationship(relationship.touch()).await?;
        info!("Relationship touched successfully");
        Ok(())
    }

    #[instrument(skip_all, fields(update_count = updates.len()))]
    pub async fn write_relationships(
        &self,
        updates: Vec<RelationshipUpdate>,
    ) -> Result<(), AuthzedError> {
        info!(
            update_count = updates.len(),
            "Writing multiple relationships"
        );

        let request = WriteRelationshipsRequest {
            updates,
            ..Default::default()
        };

        self.permissions()
            .await
            .write_relationships(request)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to write relationships");
                AuthzedError::WriteRelationshipError { msg: e.to_string() }
            })?;

        info!("Relationships written successfully");
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn filtered_delete(
        &self,
        relationship_filter: impl Into<RelationshipFilter>,
    ) -> Result<(), AuthzedError> {
        let relationship_filter: RelationshipFilter = relationship_filter.into();
        info!(
            resource_type = %relationship_filter.resource_type,
            relation = %relationship_filter.optional_relation,
            "Deleting relationships with filter"
        );

        let request = DeleteRelationshipsRequest {
            relationship_filter: Some(relationship_filter),
            ..Default::default()
        };

        self.permissions()
            .await
            .delete_relationships(request)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to delete relationships with filter");
                AuthzedError::DeleteRelationshipError { msg: e.to_string() }
            })?;

        info!("Filtered relationships deleted successfully");
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn write_relationship(
        &self,
        relationship_update: RelationshipUpdate,
    ) -> Result<(), AuthzedError> {
        debug!("Writing single relationship");

        let request = WriteRelationshipsRequest {
            updates: vec![relationship_update],
            ..Default::default()
        };

        self.permissions()
            .await
            .write_relationships(request)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to write relationship");
                AuthzedError::WriteRelationshipError { msg: e.to_string() }
            })?;

        debug!("Relationship written successfully");
        Ok(())
    }

    /// Read relationships matching the given filter
    #[instrument(skip_all)]
    pub async fn read_relationships(
        &self,
        relationship_filter: impl Into<RelationshipFilter>,
    ) -> Result<Vec<Relationship>, AuthzedError> {
        let relationship_filter: RelationshipFilter = relationship_filter.into();
        info!(
            resource_type = %relationship_filter.resource_type,
            relation = %relationship_filter.optional_relation,
            "Reading relationships with filter"
        );

        let request = ReadRelationshipsRequest {
            relationship_filter: Some(relationship_filter),
            ..Default::default()
        };

        let mut stream = self
            .permissions()
            .await
            .read_relationships(request)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to read relationships");
                AuthzedError::ConnectionError { msg: e.to_string() }
            })?
            .into_inner();

        let mut relationships = Vec::new();
        while let Some(response) = stream.next().await {
            let response = response.map_err(|e| {
                error!(error = %e, "Error in relationship stream");
                AuthzedError::ConnectionError { msg: e.to_string() }
            })?;
            relationships.push(response.relationship.unwrap());
        }

        info!(
            relationship_count = relationships.len(),
            "Successfully read relationships"
        );
        Ok(relationships)
    }
}

// Interceptor for adding authentication token to requests
#[derive(Clone)]
struct AuthInterceptor {
    token: String,
}

impl Interceptor for AuthInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        // Only add auth header if token is not empty
        if !self.token.is_empty() {
            let token = format!("Bearer {}", self.token);
            let metadata_value = tonic::metadata::MetadataValue::try_from(token)
                .map_err(|e| tonic::Status::internal(format!("Invalid token: {}", e)))?;
            request
                .metadata_mut()
                .insert("authorization", metadata_value);
        }
        Ok(request)
    }
}
