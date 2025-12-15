use crate::domain::permission_override::{
    PermissionOverrideError,
    entities::{CreatePermissionOverrideInput, DeletePermissionOverrideInput},
};
use std::future::Future;

pub trait PermissionOverrideRepository: Send + Sync {
    fn create(
        &self,
        input: CreatePermissionOverrideInput,
    ) -> impl Future<Output = Result<(), PermissionOverrideError>> + Send;
    fn delete(
        &self,
        input: DeletePermissionOverrideInput,
    ) -> impl Future<Output = Result<(), PermissionOverrideError>> + Send;
}

pub trait PermissionOverrideService: Send + Sync {
    fn create(
        &self,
        input: CreatePermissionOverrideInput,
    ) -> impl Future<Output = Result<(), PermissionOverrideError>> + Send;
    fn delete(
        &self,
        input: DeletePermissionOverrideInput,
    ) -> impl Future<Output = Result<(), PermissionOverrideError>> + Send;
}
