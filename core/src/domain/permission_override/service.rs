use crate::domain::{
    channel::port::ChannelRepository,
    common::service::Service,
    permission_override::{
        PermissionOverrideError,
        entities::{CreatePermissionOverrideInput, DeletePermissionOverrideInput},
        port::{PermissionOverrideRepository, PermissionOverrideService},
    },
    role::port::RoleRepository,
    server::port::ServerRepository,
};
use tracing::{info, instrument};

impl<S, C, R, P> PermissionOverrideService for Service<S, C, R, P>
where
    S: ServerRepository,
    C: ChannelRepository,
    R: RoleRepository,
    P: PermissionOverrideRepository,
{
    #[instrument(skip(self), fields(override_id = %input.override_id, channel_id = %input.channel_id))]
    async fn create(
        &self,
        input: CreatePermissionOverrideInput,
    ) -> Result<(), PermissionOverrideError> {
        info!(
            override_id = %input.override_id,
            channel_id = %input.channel_id,
            permission_bitmask = %input.permission_bitmask,
            is_allow = %input.is_allow,
            "Creating permission override in domain service"
        );
        let result = self.permission_override_repository.create(input).await;
        match &result {
            Ok(_) => info!("Permission override created successfully in domain service"),
            Err(e) => info!(error = ?e, "Failed to create permission override in domain service"),
        }
        result
    }

    #[instrument(skip(self), fields(override_id = %input.override_id))]
    async fn delete(
        &self,
        input: DeletePermissionOverrideInput,
    ) -> Result<(), PermissionOverrideError> {
        info!(
            override_id = %input.override_id,
            "Deleting permission override in domain service"
        );
        let result = self.permission_override_repository.delete(input).await;
        match &result {
            Ok(_) => info!("Permission override deleted successfully in domain service"),
            Err(e) => info!(error = ?e, "Failed to delete permission override in domain service"),
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        channel::{
            ChannelError,
            entities::{CreateChannelInput, DeleteChannelInput},
            port::ChannelRepository,
        },
        permission_override::entities::OverrideTarget,
        role::{
            RoleError,
            entities::{AssignMemberInput, CreateRoleInput, DeleteRoleInput, RemoveMemberInput},
            port::RoleRepository,
        },
        server::{
            ServerError,
            entities::{CreateServerInput, DeleteServerInput},
            port::ServerRepository,
        },
    };
    use std::sync::{Arc, Mutex};

    // Mock PermissionOverrideRepository for testing
    #[derive(Clone)]
    struct MockPermissionOverrideRepository {
        should_fail: Arc<Mutex<bool>>,
        error_message: Arc<Mutex<String>>,
        call_count: Arc<Mutex<usize>>,
        last_create_input: Arc<Mutex<Option<CreatePermissionOverrideInput>>>,
        last_delete_input: Arc<Mutex<Option<DeletePermissionOverrideInput>>>,
    }

    impl MockPermissionOverrideRepository {
        fn new() -> Self {
            Self {
                should_fail: Arc::new(Mutex::new(false)),
                error_message: Arc::new(Mutex::new(String::new())),
                call_count: Arc::new(Mutex::new(0)),
                last_create_input: Arc::new(Mutex::new(None)),
                last_delete_input: Arc::new(Mutex::new(None)),
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

        fn get_last_create_input(&self) -> Option<CreatePermissionOverrideInput> {
            self.last_create_input.lock().unwrap().clone()
        }

        fn get_last_delete_input(&self) -> Option<DeletePermissionOverrideInput> {
            self.last_delete_input.lock().unwrap().clone()
        }
    }

    impl PermissionOverrideRepository for MockPermissionOverrideRepository {
        async fn create(
            &self,
            input: CreatePermissionOverrideInput,
        ) -> Result<(), PermissionOverrideError> {
            *self.call_count.lock().unwrap() += 1;
            *self.last_create_input.lock().unwrap() = Some(input);

            if *self.should_fail.lock().unwrap() {
                let msg = self.error_message.lock().unwrap().clone();
                Err(PermissionOverrideError::CreateOverrideError { msg })
            } else {
                Ok(())
            }
        }

        async fn delete(
            &self,
            input: DeletePermissionOverrideInput,
        ) -> Result<(), PermissionOverrideError> {
            *self.call_count.lock().unwrap() += 1;
            *self.last_delete_input.lock().unwrap() = Some(input);

            if *self.should_fail.lock().unwrap() {
                let msg = self.error_message.lock().unwrap().clone();
                Err(PermissionOverrideError::DeleteOverrideError { msg })
            } else {
                Ok(())
            }
        }
    }

    // Stub repositories for other dependencies
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

    #[derive(Clone)]
    struct MockChannelRepository;

    impl ChannelRepository for MockChannelRepository {
        async fn create(&self, _input: CreateChannelInput) -> Result<(), ChannelError> {
            Ok(())
        }

        async fn delete(&self, _input: DeleteChannelInput) -> Result<(), ChannelError> {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct MockRoleRepository;

    impl RoleRepository for MockRoleRepository {
        async fn create(&self, _input: CreateRoleInput) -> Result<(), RoleError> {
            Ok(())
        }

        async fn delete(&self, _input: DeleteRoleInput) -> Result<(), RoleError> {
            Ok(())
        }

        async fn assign_member(&self, _input: AssignMemberInput) -> Result<(), RoleError> {
            Ok(())
        }

        async fn remove_member(&self, _input: RemoveMemberInput) -> Result<(), RoleError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_override_success() {
        // Arrange
        let mock_override_repo = MockPermissionOverrideRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            MockRoleRepository,
            mock_override_repo.clone(),
        );

        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0xC0, // ViewChannels | SendMessages
            is_allow: true,
            target: OverrideTarget::User("user_789".to_string()),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(mock_override_repo.get_call_count(), 1);

        let last_input = mock_override_repo.get_last_create_input().unwrap();
        assert_eq!(last_input.override_id, "override_123");
        assert_eq!(last_input.channel_id, "channel_456");
        assert_eq!(last_input.permission_bitmask, 0xC0);
        assert!(last_input.is_allow);
    }

    #[tokio::test]
    async fn test_create_override_failure() {
        // Arrange
        let mock_override_repo =
            MockPermissionOverrideRepository::new().with_failure("AuthZed connection failed");
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            MockRoleRepository,
            mock_override_repo.clone(),
        );

        let input = CreatePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0x80,
            is_allow: false,
            target: OverrideTarget::Role("role_123".to_string()),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(mock_override_repo.get_call_count(), 1);

        if let Err(PermissionOverrideError::CreateOverrideError { msg }) = result {
            assert_eq!(msg, "AuthZed connection failed");
        } else {
            panic!("Expected CreateOverrideError");
        }
    }

    #[tokio::test]
    async fn test_create_override_with_different_inputs() {
        // Arrange
        let mock_override_repo = MockPermissionOverrideRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            MockRoleRepository,
            mock_override_repo.clone(),
        );

        let input = CreatePermissionOverrideInput {
            override_id: "test_override".to_string(),
            channel_id: "test_channel".to_string(),
            permission_bitmask: 0x860, // Multiple channel permissions
            is_allow: true,
            target: OverrideTarget::User("test_user".to_string()),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_ok());

        let last_input = mock_override_repo.get_last_create_input().unwrap();
        assert_eq!(last_input.override_id, "test_override");
        assert_eq!(last_input.channel_id, "test_channel");
        assert_eq!(last_input.permission_bitmask, 0x860);
        assert!(last_input.is_allow);
    }

    #[tokio::test]
    async fn test_create_override_propagates_error() {
        // Arrange
        let error_msg = "Permission denied";
        let mock_override_repo = MockPermissionOverrideRepository::new().with_failure(error_msg);
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            MockRoleRepository,
            mock_override_repo,
        );

        let input = CreatePermissionOverrideInput {
            override_id: "override_xyz".to_string(),
            channel_id: "channel_abc".to_string(),
            permission_bitmask: 0x40,
            is_allow: false,
            target: OverrideTarget::Role("role_456".to_string()),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(PermissionOverrideError::CreateOverrideError { msg }) => {
                assert_eq!(msg, error_msg);
            }
            _ => panic!("Expected CreateOverrideError"),
        }
    }

    #[tokio::test]
    async fn test_create_override_multiple_calls() {
        // Arrange
        let mock_override_repo = MockPermissionOverrideRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            MockRoleRepository,
            mock_override_repo.clone(),
        );

        // Act - create multiple overrides
        let input1 = CreatePermissionOverrideInput {
            override_id: "override_1".to_string(),
            channel_id: "channel_1".to_string(),
            permission_bitmask: 0x80,
            is_allow: true,
            target: OverrideTarget::User("user_1".to_string()),
        };
        let result1 = service.create(input1).await;

        let input2 = CreatePermissionOverrideInput {
            override_id: "override_2".to_string(),
            channel_id: "channel_2".to_string(),
            permission_bitmask: 0x40,
            is_allow: false,
            target: OverrideTarget::Role("role_2".to_string()),
        };
        let result2 = service.create(input2).await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(mock_override_repo.get_call_count(), 2);

        // Last input should be the second one
        let last_input = mock_override_repo.get_last_create_input().unwrap();
        assert_eq!(last_input.override_id, "override_2");
        assert_eq!(last_input.channel_id, "channel_2");
        assert_eq!(last_input.permission_bitmask, 0x40);
        assert!(!last_input.is_allow);
    }

    #[tokio::test]
    async fn test_delete_override_success() {
        // Arrange
        let mock_override_repo = MockPermissionOverrideRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            MockRoleRepository,
            mock_override_repo.clone(),
        );

        let input = DeletePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0x80,
            is_allow: true,
            target: OverrideTarget::User("user_789".to_string()),
        };

        // Act
        let result = service.delete(input).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(mock_override_repo.get_call_count(), 1);

        let last_input = mock_override_repo.get_last_delete_input().unwrap();
        assert_eq!(last_input.override_id, "override_123");
        assert_eq!(last_input.channel_id, "channel_456");
    }

    #[tokio::test]
    async fn test_delete_override_failure() {
        // Arrange
        let mock_override_repo =
            MockPermissionOverrideRepository::new().with_failure("Override not found");
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            MockRoleRepository,
            mock_override_repo.clone(),
        );

        let input = DeletePermissionOverrideInput {
            override_id: "override_123".to_string(),
            channel_id: "channel_456".to_string(),
            permission_bitmask: 0x80,
            is_allow: true,
            target: OverrideTarget::User("user_789".to_string()),
        };

        // Act
        let result = service.delete(input).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(mock_override_repo.get_call_count(), 1);

        if let Err(PermissionOverrideError::DeleteOverrideError { msg }) = result {
            assert_eq!(msg, "Override not found");
        } else {
            panic!("Expected DeleteOverrideError");
        }
    }

    #[tokio::test]
    async fn test_multiple_override_operations() {
        // Arrange
        let mock_override_repo = MockPermissionOverrideRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            MockRoleRepository,
            mock_override_repo.clone(),
        );

        // Act - perform multiple operations
        let create_result = service
            .create(CreatePermissionOverrideInput {
                override_id: "override_1".to_string(),
                channel_id: "channel_1".to_string(),
                permission_bitmask: 0xC0,
                is_allow: true,
                target: OverrideTarget::User("user_1".to_string()),
            })
            .await;

        let delete_result = service
            .delete(DeletePermissionOverrideInput {
                override_id: "override_2".to_string(),
                channel_id: "channel_2".to_string(),
                permission_bitmask: 0x80,
                is_allow: false,
                target: OverrideTarget::Role("role_1".to_string()),
            })
            .await;

        // Assert
        assert!(create_result.is_ok());
        assert!(delete_result.is_ok());
        assert_eq!(mock_override_repo.get_call_count(), 2);
    }

    #[tokio::test]
    async fn test_create_override_with_role_target() {
        // Arrange
        let mock_override_repo = MockPermissionOverrideRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            MockRoleRepository,
            mock_override_repo.clone(),
        );

        let input = CreatePermissionOverrideInput {
            override_id: "override_role".to_string(),
            channel_id: "channel_789".to_string(),
            permission_bitmask: 0x460, // Multiple permissions
            is_allow: false,
            target: OverrideTarget::Role("role_999".to_string()),
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_ok());
        let last_input = mock_override_repo.get_last_create_input().unwrap();

        match last_input.target {
            OverrideTarget::Role(role_id) => assert_eq!(role_id, "role_999"),
            _ => panic!("Expected Role target"),
        }
    }
}
