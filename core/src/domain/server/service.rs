use crate::domain::{
    common::service::Service,
    server::{
        ServerError,
        entities::CreateServerInput,
        port::{ServerRepository, ServerService},
    },
};
use tracing::{info, instrument};

impl<S> ServerService for Service<S>
where
    S: ServerRepository,
{
    #[instrument(skip(self), fields(server_id = %input.server_id, owner_id = %input.owner_id))]
    async fn create(&self, input: CreateServerInput) -> Result<(), ServerError> {
        info!(
            server_id = %input.server_id,
            owner_id = %input.owner_id,
            "Creating server in domain service"
        );
        let result = self.server_repository.create(input).await;
        match &result {
            Ok(_) => info!("Server created successfully in domain service"),
            Err(e) => info!(error = ?e, "Failed to create server in domain service"),
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Simple mock repository for testing
    #[derive(Clone)]
    struct MockServerRepository {
        should_fail: Arc<Mutex<bool>>,
        error_message: Arc<Mutex<String>>,
        call_count: Arc<Mutex<usize>>,
        last_input: Arc<Mutex<Option<CreateServerInput>>>,
    }

    impl MockServerRepository {
        fn new() -> Self {
            Self {
                should_fail: Arc::new(Mutex::new(false)),
                error_message: Arc::new(Mutex::new(String::new())),
                call_count: Arc::new(Mutex::new(0)),
                last_input: Arc::new(Mutex::new(None)),
            }
        }

        fn with_failure(self, error_msg: &str) -> Self {
            *self.should_fail.lock().unwrap() = true;
            *self.error_message.lock().unwrap() = error_msg.to_string();
            self
        }

        fn get_call_count(&self) -> usize {
            *self.call_count.lock().unwrap()
        }

        fn get_last_input(&self) -> Option<CreateServerInput> {
            self.last_input.lock().unwrap().clone()
        }
    }

    impl ServerRepository for MockServerRepository {
        async fn create(&self, input: CreateServerInput) -> Result<(), ServerError> {
            *self.call_count.lock().unwrap() += 1;
            *self.last_input.lock().unwrap() = Some(input);

            if *self.should_fail.lock().unwrap() {
                let msg = self.error_message.lock().unwrap().clone();
                Err(ServerError::CreateServerError { msg })
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_create_server_success() {
        // Arrange
        let mock_repo = MockServerRepository::new();
        let service = Service::new(mock_repo.clone());

        let input = CreateServerInput {
            server_id: "server_123".to_string(),
            owner_id: "owner_456".to_string(),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(mock_repo.get_call_count(), 1);

        let last_input = mock_repo.get_last_input().unwrap();
        assert_eq!(last_input.server_id, "server_123");
        assert_eq!(last_input.owner_id, "owner_456");
    }

    #[tokio::test]
    async fn test_create_server_failure() {
        // Arrange
        let mock_repo = MockServerRepository::new().with_failure("Database connection failed");
        let service = Service::new(mock_repo.clone());

        let input = CreateServerInput {
            server_id: "server_123".to_string(),
            owner_id: "owner_456".to_string(),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(mock_repo.get_call_count(), 1);

        if let Err(ServerError::CreateServerError { msg }) = result {
            assert_eq!(msg, "Database connection failed");
        } else {
            panic!("Expected CreateServerError");
        }
    }

    #[tokio::test]
    async fn test_create_server_with_different_inputs() {
        // Arrange
        let mock_repo = MockServerRepository::new();
        let service = Service::new(mock_repo.clone());

        let input = CreateServerInput {
            server_id: "test_server".to_string(),
            owner_id: "test_owner".to_string(),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_ok());

        let last_input = mock_repo.get_last_input().unwrap();
        assert_eq!(last_input.server_id, "test_server");
        assert_eq!(last_input.owner_id, "test_owner");
    }

    #[tokio::test]
    async fn test_create_server_propagates_error() {
        // Arrange
        let error_msg = "Permission denied";
        let mock_repo = MockServerRepository::new().with_failure(error_msg);
        let service = Service::new(mock_repo);

        let input = CreateServerInput {
            server_id: "server_xyz".to_string(),
            owner_id: "owner_abc".to_string(),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServerError::CreateServerError { msg }) => {
                assert_eq!(msg, error_msg);
            }
            _ => panic!("Expected CreateServerError"),
        }
    }

    #[tokio::test]
    async fn test_create_server_multiple_calls() {
        // Arrange
        let mock_repo = MockServerRepository::new();
        let service = Service::new(mock_repo.clone());

        // Act - create multiple servers
        let input1 = CreateServerInput {
            server_id: "server_1".to_string(),
            owner_id: "owner_1".to_string(),
        };
        let result1 = service.create(input1).await;

        let input2 = CreateServerInput {
            server_id: "server_2".to_string(),
            owner_id: "owner_2".to_string(),
        };
        let result2 = service.create(input2).await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(mock_repo.get_call_count(), 2);

        // Last input should be the second one
        let last_input = mock_repo.get_last_input().unwrap();
        assert_eq!(last_input.server_id, "server_2");
        assert_eq!(last_input.owner_id, "owner_2");
    }
}
