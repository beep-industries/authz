use crate::domain::{
    channel::port::ChannelRepository,
    common::service::Service,
    permission_override::port::PermissionOverrideRepository,
    role::{
        RoleError,
        entities::{AssignMemberInput, CreateRoleInput, DeleteRoleInput, RemoveMemberInput},
        port::{RoleRepository, RoleService},
    },
    server::port::ServerRepository,
};
use tracing::{info, instrument};

impl<S, C, R, P> RoleService for Service<S, C, R, P>
where
    S: ServerRepository,
    C: ChannelRepository,
    R: RoleRepository,
    P: PermissionOverrideRepository,
{
    #[instrument(skip(self), fields(role_id = %input.role_id, server_id = %input.server_id, permissions_bitmask = %input.permissions_bitmask))]
    async fn create(&self, input: CreateRoleInput) -> Result<(), RoleError> {
        info!(
            role_id = %input.role_id,
            server_id = %input.server_id,
            permissions_bitmask = %input.permissions_bitmask,
            "Creating role in domain service"
        );
        let result = self.role_repository.create(input).await;
        match &result {
            Ok(_) => info!("Role created successfully in domain service"),
            Err(e) => info!(error = ?e, "Failed to create role in domain service"),
        }
        result
    }

    #[instrument(skip(self), fields(role_id = %input.role_id))]
    async fn delete(&self, input: DeleteRoleInput) -> Result<(), RoleError> {
        info!(
            role_id = %input.role_id,
            "Deleting role in domain service"
        );
        let result = self.role_repository.delete(input).await;
        match &result {
            Ok(_) => info!("Role deleted successfully in domain service"),
            Err(e) => info!(error = ?e, "Failed to delete role in domain service"),
        }
        result
    }

    #[instrument(skip(self), fields(user_id = %input.user_id, role_id = %input.role_id))]
    async fn assign_member(&self, input: AssignMemberInput) -> Result<(), RoleError> {
        info!(
            user_id = %input.user_id,
            role_id = %input.role_id,
            "Assigning member to role in domain service"
        );
        let result = self.role_repository.assign_member(input).await;
        match &result {
            Ok(_) => info!("Member assigned successfully in domain service"),
            Err(e) => info!(error = ?e, "Failed to assign member in domain service"),
        }
        result
    }

    #[instrument(skip(self), fields(user_id = %input.user_id, role_id = %input.role_id))]
    async fn remove_member(&self, input: RemoveMemberInput) -> Result<(), RoleError> {
        info!(
            user_id = %input.user_id,
            role_id = %input.role_id,
            "Removing member from role in domain service"
        );
        let result = self.role_repository.remove_member(input).await;
        match &result {
            Ok(_) => info!("Member removed successfully in domain service"),
            Err(e) => info!(error = ?e, "Failed to remove member in domain service"),
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
        permission_override::{
            PermissionOverrideError,
            entities::{CreatePermissionOverrideInput, DeletePermissionOverrideInput},
            port::PermissionOverrideRepository,
        },
        server::{
            ServerError,
            entities::{CreateServerInput, DeleteServerInput},
            port::ServerRepository,
        },
    };
    use std::sync::{Arc, Mutex};

    // Mock RoleRepository for testing
    #[derive(Clone)]
    struct MockRoleRepository {
        should_fail: Arc<Mutex<bool>>,
        error_message: Arc<Mutex<String>>,
        call_count: Arc<Mutex<usize>>,
        last_create_input: Arc<Mutex<Option<CreateRoleInput>>>,
        last_assign_input: Arc<Mutex<Option<AssignMemberInput>>>,
        last_remove_input: Arc<Mutex<Option<RemoveMemberInput>>>,
    }

    impl MockRoleRepository {
        fn new() -> Self {
            Self {
                should_fail: Arc::new(Mutex::new(false)),
                error_message: Arc::new(Mutex::new(String::new())),
                call_count: Arc::new(Mutex::new(0)),
                last_create_input: Arc::new(Mutex::new(None)),
                last_assign_input: Arc::new(Mutex::new(None)),
                last_remove_input: Arc::new(Mutex::new(None)),
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

        fn get_last_create_input(&self) -> Option<CreateRoleInput> {
            self.last_create_input.lock().unwrap().clone()
        }

        fn get_last_assign_input(&self) -> Option<AssignMemberInput> {
            self.last_assign_input.lock().unwrap().clone()
        }

        fn get_last_remove_input(&self) -> Option<RemoveMemberInput> {
            self.last_remove_input.lock().unwrap().clone()
        }
    }

    impl RoleRepository for MockRoleRepository {
        async fn create(&self, input: CreateRoleInput) -> Result<(), RoleError> {
            *self.call_count.lock().unwrap() += 1;
            *self.last_create_input.lock().unwrap() = Some(input);

            if *self.should_fail.lock().unwrap() {
                let msg = self.error_message.lock().unwrap().clone();
                Err(RoleError::CreateRoleError { msg })
            } else {
                Ok(())
            }
        }

        async fn delete(&self, _input: DeleteRoleInput) -> Result<(), RoleError> {
            *self.call_count.lock().unwrap() += 1;
            if *self.should_fail.lock().unwrap() {
                let msg = self.error_message.lock().unwrap().clone();
                Err(RoleError::DeleteRoleError { msg })
            } else {
                Ok(())
            }
        }

        async fn assign_member(&self, input: AssignMemberInput) -> Result<(), RoleError> {
            *self.call_count.lock().unwrap() += 1;
            *self.last_assign_input.lock().unwrap() = Some(input);

            if *self.should_fail.lock().unwrap() {
                let msg = self.error_message.lock().unwrap().clone();
                Err(RoleError::AssignMemberError { msg })
            } else {
                Ok(())
            }
        }

        async fn remove_member(&self, input: RemoveMemberInput) -> Result<(), RoleError> {
            *self.call_count.lock().unwrap() += 1;
            *self.last_remove_input.lock().unwrap() = Some(input);

            if *self.should_fail.lock().unwrap() {
                let msg = self.error_message.lock().unwrap().clone();
                Err(RoleError::RemoveMemberError { msg })
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
    struct MockPermissionOverrideRepository;

    impl PermissionOverrideRepository for MockPermissionOverrideRepository {
        async fn create(
            &self,
            _input: CreatePermissionOverrideInput,
        ) -> Result<(), PermissionOverrideError> {
            Ok(())
        }

        async fn delete(
            &self,
            _input: DeletePermissionOverrideInput,
        ) -> Result<(), PermissionOverrideError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_role_success() {
        // Arrange
        let mock_role_repo = MockRoleRepository::new();
        let mock_server_repo = MockServerRepository;
        let mock_channel_repo = MockChannelRepository;
        let mock_override_repo = MockPermissionOverrideRepository;
        let service = Service::new(
            mock_server_repo,
            mock_channel_repo,
            mock_role_repo.clone(),
            mock_override_repo,
        );

        let input = CreateRoleInput {
            role_id: "role_123".to_string(),
            server_id: "server_456".to_string(),
            permissions_bitmask: 0x88,
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(mock_role_repo.get_call_count(), 1);

        let last_input = mock_role_repo.get_last_create_input().unwrap();
        assert_eq!(last_input.role_id, "role_123");
        assert_eq!(last_input.server_id, "server_456");
        assert_eq!(last_input.permissions_bitmask, 0x88);
    }

    #[tokio::test]
    async fn test_create_role_failure() {
        // Arrange
        let mock_role_repo = MockRoleRepository::new().with_failure("AuthZed connection failed");
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            mock_role_repo.clone(),
            MockPermissionOverrideRepository,
        );

        let input = CreateRoleInput {
            role_id: "role_123".to_string(),
            server_id: "server_456".to_string(),
            permissions_bitmask: 0x1,
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(mock_role_repo.get_call_count(), 1);

        if let Err(RoleError::CreateRoleError { msg }) = result {
            assert_eq!(msg, "AuthZed connection failed");
        } else {
            panic!("Expected CreateRoleError");
        }
    }

    #[tokio::test]
    async fn test_create_role_with_different_inputs() {
        // Arrange
        let mock_role_repo = MockRoleRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            mock_role_repo.clone(),
            MockPermissionOverrideRepository,
        );

        let input = CreateRoleInput {
            role_id: "test_role".to_string(),
            server_id: "test_server".to_string(),
            permissions_bitmask: 0xFFF,
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_ok());

        let last_input = mock_role_repo.get_last_create_input().unwrap();
        assert_eq!(last_input.role_id, "test_role");
        assert_eq!(last_input.server_id, "test_server");
        assert_eq!(last_input.permissions_bitmask, 0xFFF);
    }

    #[tokio::test]
    async fn test_create_role_propagates_error() {
        // Arrange
        let error_msg = "Permission denied";
        let mock_role_repo = MockRoleRepository::new().with_failure(error_msg);
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            mock_role_repo,
            MockPermissionOverrideRepository,
        );

        let input = CreateRoleInput {
            role_id: "role_xyz".to_string(),
            server_id: "server_abc".to_string(),
            permissions_bitmask: 0x2,
        };

        // Act
        let result = service.create(input).await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(RoleError::CreateRoleError { msg }) => {
                assert_eq!(msg, error_msg);
            }
            _ => panic!("Expected CreateRoleError"),
        }
    }

    #[tokio::test]
    async fn test_create_role_multiple_calls() {
        // Arrange
        let mock_role_repo = MockRoleRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            mock_role_repo.clone(),
            MockPermissionOverrideRepository,
        );

        // Act - create multiple roles
        let input1 = CreateRoleInput {
            role_id: "role_1".to_string(),
            server_id: "server_1".to_string(),
            permissions_bitmask: 0x1,
        };
        let result1 = service.create(input1).await;

        let input2 = CreateRoleInput {
            role_id: "role_2".to_string(),
            server_id: "server_2".to_string(),
            permissions_bitmask: 0x2,
        };
        let result2 = service.create(input2).await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(mock_role_repo.get_call_count(), 2);

        // Last input should be the second one
        let last_input = mock_role_repo.get_last_create_input().unwrap();
        assert_eq!(last_input.role_id, "role_2");
        assert_eq!(last_input.server_id, "server_2");
        assert_eq!(last_input.permissions_bitmask, 0x2);
    }

    #[tokio::test]
    async fn test_delete_role_success() {
        // Arrange
        let mock_role_repo = MockRoleRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            mock_role_repo.clone(),
            MockPermissionOverrideRepository,
        );

        let input = DeleteRoleInput {
            role_id: "role_123".to_string(),
        };

        // Act
        let result = service.delete(input).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(mock_role_repo.get_call_count(), 1);
    }

    #[tokio::test]
    async fn test_assign_member_success() {
        // Arrange
        let mock_role_repo = MockRoleRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            mock_role_repo.clone(),
            MockPermissionOverrideRepository,
        );

        let input = AssignMemberInput {
            user_id: "user_123".to_string(),
            role_id: "role_456".to_string(),
        };

        // Act
        let result = service.assign_member(input).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(mock_role_repo.get_call_count(), 1);

        let last_input = mock_role_repo.get_last_assign_input().unwrap();
        assert_eq!(last_input.user_id, "user_123");
        assert_eq!(last_input.role_id, "role_456");
    }

    #[tokio::test]
    async fn test_assign_member_failure() {
        // Arrange
        let mock_role_repo = MockRoleRepository::new().with_failure("User not found");
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            mock_role_repo.clone(),
            MockPermissionOverrideRepository,
        );

        let input = AssignMemberInput {
            user_id: "user_123".to_string(),
            role_id: "role_456".to_string(),
        };

        // Act
        let result = service.assign_member(input).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(mock_role_repo.get_call_count(), 1);

        if let Err(RoleError::AssignMemberError { msg }) = result {
            assert_eq!(msg, "User not found");
        } else {
            panic!("Expected AssignMemberError");
        }
    }

    #[tokio::test]
    async fn test_remove_member_success() {
        // Arrange
        let mock_role_repo = MockRoleRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            mock_role_repo.clone(),
            MockPermissionOverrideRepository,
        );

        let input = RemoveMemberInput {
            user_id: "user_789".to_string(),
            role_id: "role_012".to_string(),
        };

        // Act
        let result = service.remove_member(input).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(mock_role_repo.get_call_count(), 1);

        let last_input = mock_role_repo.get_last_remove_input().unwrap();
        assert_eq!(last_input.user_id, "user_789");
        assert_eq!(last_input.role_id, "role_012");
    }

    #[tokio::test]
    async fn test_multiple_role_operations() {
        // Arrange
        let mock_role_repo = MockRoleRepository::new();
        let service = Service::new(
            MockServerRepository,
            MockChannelRepository,
            mock_role_repo.clone(),
            MockPermissionOverrideRepository,
        );

        // Act - perform multiple operations
        let create_result = service
            .create(CreateRoleInput {
                role_id: "role_1".to_string(),
                server_id: "server_1".to_string(),
                permissions_bitmask: 0x88,
            })
            .await;

        let assign_result = service
            .assign_member(AssignMemberInput {
                user_id: "user_1".to_string(),
                role_id: "role_1".to_string(),
            })
            .await;

        let remove_result = service
            .remove_member(RemoveMemberInput {
                user_id: "user_2".to_string(),
                role_id: "role_1".to_string(),
            })
            .await;

        let delete_result = service
            .delete(DeleteRoleInput {
                role_id: "role_1".to_string(),
            })
            .await;

        // Assert
        assert!(create_result.is_ok());
        assert!(assign_result.is_ok());
        assert!(remove_result.is_ok());
        assert!(delete_result.is_ok());
        assert_eq!(mock_role_repo.get_call_count(), 4);
    }
}
