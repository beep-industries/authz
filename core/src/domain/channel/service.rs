use crate::domain::{
    channel::{
        ChannelError,
        entities::{CreateChannelInput, DeleteChannelInput},
        port::{ChannelRepository, ChannelService},
    },
    common::service::Service,
    server::port::ServerRepository,
};
use tracing::{info, instrument};

impl<S, C> ChannelService for Service<S, C>
where
    S: ServerRepository,
    C: ChannelRepository,
{
    #[instrument(skip(self), fields(channel_id = %input.channel_id, server_id = %input.server_id))]
    async fn create(&self, input: CreateChannelInput) -> Result<(), ChannelError> {
        info!(
            channel_id = %input.channel_id,
            server_id = %input.server_id,
            "Creating channel in domain service"
        );
        let result = self.channel_repository.create(input).await;
        match &result {
            Ok(_) => info!("Channel created successfully in domain service"),
            Err(e) => info!(error = ?e, "Failed to create channel in domain service"),
        }
        result
    }

    #[instrument(skip(self), fields(channel_id = %input.channel_id))]
    async fn delete(&self, input: DeleteChannelInput) -> Result<(), ChannelError> {
        info!(
            channel_id = %input.channel_id,
            "Deleting channel in domain service"
        );
        let result = self.channel_repository.delete(input).await;
        match &result {
            Ok(_) => info!("Channel deleted successfully in domain service"),
            Err(e) => info!(error = ?e, "Failed to delete channel in domain service"),
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::server::{
        ServerError,
        entities::{CreateServerInput, DeleteServerInput},
        port::ServerRepository,
    };
    use std::sync::{Arc, Mutex};

    // Simple mock repository for testing
    #[derive(Clone)]
    struct MockChannelRepository {
        should_fail_create: Arc<Mutex<bool>>,
        should_fail_delete: Arc<Mutex<bool>>,
        create_error_message: Arc<Mutex<String>>,
        delete_error_message: Arc<Mutex<String>>,
        create_call_count: Arc<Mutex<usize>>,
        delete_call_count: Arc<Mutex<usize>>,
        last_create_input: Arc<Mutex<Option<CreateChannelInput>>>,
        last_delete_input: Arc<Mutex<Option<DeleteChannelInput>>>,
    }

    impl MockChannelRepository {
        fn new() -> Self {
            Self {
                should_fail_create: Arc::new(Mutex::new(false)),
                should_fail_delete: Arc::new(Mutex::new(false)),
                create_error_message: Arc::new(Mutex::new(String::new())),
                delete_error_message: Arc::new(Mutex::new(String::new())),
                create_call_count: Arc::new(Mutex::new(0)),
                delete_call_count: Arc::new(Mutex::new(0)),
                last_create_input: Arc::new(Mutex::new(None)),
                last_delete_input: Arc::new(Mutex::new(None)),
            }
        }

        fn with_create_failure(self, error_msg: &str) -> Self {
            *self.should_fail_create.lock().unwrap() = true;
            *self.create_error_message.lock().unwrap() = error_msg.to_string();
            self
        }

        fn with_delete_failure(self, error_msg: &str) -> Self {
            *self.should_fail_delete.lock().unwrap() = true;
            *self.delete_error_message.lock().unwrap() = error_msg.to_string();
            self
        }

        fn get_create_call_count(&self) -> usize {
            *self.create_call_count.lock().unwrap()
        }

        fn get_delete_call_count(&self) -> usize {
            *self.delete_call_count.lock().unwrap()
        }

        fn get_last_create_input(&self) -> Option<CreateChannelInput> {
            self.last_create_input.lock().unwrap().clone()
        }

        fn get_last_delete_input(&self) -> Option<DeleteChannelInput> {
            self.last_delete_input.lock().unwrap().clone()
        }
    }

    impl ChannelRepository for MockChannelRepository {
        async fn create(&self, input: CreateChannelInput) -> Result<(), ChannelError> {
            *self.create_call_count.lock().unwrap() += 1;
            *self.last_create_input.lock().unwrap() = Some(input);

            if *self.should_fail_create.lock().unwrap() {
                let msg = self.create_error_message.lock().unwrap().clone();
                Err(ChannelError::CreateChannelError { msg })
            } else {
                Ok(())
            }
        }

        async fn delete(&self, input: DeleteChannelInput) -> Result<(), ChannelError> {
            *self.delete_call_count.lock().unwrap() += 1;
            *self.last_delete_input.lock().unwrap() = Some(input);

            if *self.should_fail_delete.lock().unwrap() {
                let msg = self.delete_error_message.lock().unwrap().clone();
                Err(ChannelError::DeleteChannelError { msg })
            } else {
                Ok(())
            }
        }
    }

    #[derive(Clone)]
    struct MockServerRepository;

    impl ServerRepository for MockServerRepository {
        async fn create(&self, _input: CreateServerInput) -> Result<(), ServerError> {
            Ok(())
        }

        async fn delete(&self, _input: DeleteServerInput) -> Result<(), ServerError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_channel_success() {
        // Arrange
        let mock_repo = MockChannelRepository::new();
        let mock_server_repo = MockServerRepository;
        let service = Service::new(mock_server_repo, mock_repo.clone());

        let input = CreateChannelInput {
            channel_id: "channel_123".to_string(),
            server_id: "server_456".to_string(),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(mock_repo.get_create_call_count(), 1);

        let last_input = mock_repo.get_last_create_input().unwrap();
        assert_eq!(last_input.channel_id, "channel_123");
        assert_eq!(last_input.server_id, "server_456");
    }

    #[tokio::test]
    async fn test_create_channel_failure() {
        // Arrange
        let mock_repo =
            MockChannelRepository::new().with_create_failure("Database connection failed");
        let mock_server_repo = MockServerRepository;
        let service = Service::new(mock_server_repo, mock_repo.clone());

        let input = CreateChannelInput {
            channel_id: "channel_123".to_string(),
            server_id: "server_456".to_string(),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(mock_repo.get_create_call_count(), 1);

        if let Err(ChannelError::CreateChannelError { msg }) = result {
            assert_eq!(msg, "Database connection failed");
        } else {
            panic!("Expected CreateChannelError");
        }
    }

    #[tokio::test]
    async fn test_delete_channel_success() {
        // Arrange
        let mock_repo = MockChannelRepository::new();
        let mock_server_repo = MockServerRepository;
        let service = Service::new(mock_server_repo, mock_repo.clone());

        let input = DeleteChannelInput {
            channel_id: "channel_123".to_string(),
        };

        // Act
        let result = service.delete(input).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(mock_repo.get_delete_call_count(), 1);

        let last_input = mock_repo.get_last_delete_input().unwrap();
        assert_eq!(last_input.channel_id, "channel_123");
    }

    #[tokio::test]
    async fn test_delete_channel_failure() {
        // Arrange
        let mock_repo = MockChannelRepository::new().with_delete_failure("Permission denied");
        let mock_server_repo = MockServerRepository;
        let service = Service::new(mock_server_repo, mock_repo.clone());

        let input = DeleteChannelInput {
            channel_id: "channel_123".to_string(),
        };

        // Act
        let result = service.delete(input).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(mock_repo.get_delete_call_count(), 1);

        if let Err(ChannelError::DeleteChannelError { msg }) = result {
            assert_eq!(msg, "Permission denied");
        } else {
            panic!("Expected DeleteChannelError");
        }
    }

    #[tokio::test]
    async fn test_create_channel_propagates_error() {
        // Arrange
        let error_msg = "SpiceDB unavailable";
        let mock_repo = MockChannelRepository::new().with_create_failure(error_msg);
        let mock_server_repo = MockServerRepository;
        let service = Service::new(mock_server_repo, mock_repo);

        let input = CreateChannelInput {
            channel_id: "channel_xyz".to_string(),
            server_id: "server_abc".to_string(),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(ChannelError::CreateChannelError { msg }) => {
                assert_eq!(msg, error_msg);
            }
            _ => panic!("Expected CreateChannelError"),
        }
    }

    #[tokio::test]
    async fn test_delete_channel_propagates_error() {
        // Arrange
        let error_msg = "Channel not found";
        let mock_repo = MockChannelRepository::new().with_delete_failure(error_msg);
        let mock_server_repo = MockServerRepository;
        let service = Service::new(mock_server_repo, mock_repo);

        let input = DeleteChannelInput {
            channel_id: "channel_xyz".to_string(),
        };

        // Act
        let result = service.delete(input).await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(ChannelError::DeleteChannelError { msg }) => {
                assert_eq!(msg, error_msg);
            }
            _ => panic!("Expected DeleteChannelError"),
        }
    }

    #[tokio::test]
    async fn test_multiple_channel_operations() {
        // Arrange
        let mock_repo = MockChannelRepository::new();
        let mock_server_repo = MockServerRepository;
        let service = Service::new(mock_server_repo, mock_repo.clone());

        // Act - create a channel
        let create_input = CreateChannelInput {
            channel_id: "channel_1".to_string(),
            server_id: "server_1".to_string(),
        };
        let create_result = service.create(create_input).await;

        // Act - delete a channel
        let delete_input = DeleteChannelInput {
            channel_id: "channel_1".to_string(),
        };
        let delete_result = service.delete(delete_input).await;

        // Assert
        assert!(create_result.is_ok());
        assert!(delete_result.is_ok());
        assert_eq!(mock_repo.get_create_call_count(), 1);
        assert_eq!(mock_repo.get_delete_call_count(), 1);

        let last_create = mock_repo.get_last_create_input().unwrap();
        assert_eq!(last_create.channel_id, "channel_1");
        assert_eq!(last_create.server_id, "server_1");

        let last_delete = mock_repo.get_last_delete_input().unwrap();
        assert_eq!(last_delete.channel_id, "channel_1");
    }
}
