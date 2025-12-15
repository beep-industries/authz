use crate::domain::role::{
    RoleError,
    entities::{AssignMemberInput, CreateRoleInput, DeleteRoleInput, RemoveMemberInput},
};
use std::future::Future;

pub trait RoleRepository: Send + Sync {
    fn create(&self, input: CreateRoleInput) -> impl Future<Output = Result<(), RoleError>> + Send;
    fn delete(&self, input: DeleteRoleInput) -> impl Future<Output = Result<(), RoleError>> + Send;
    fn assign_member(
        &self,
        input: AssignMemberInput,
    ) -> impl Future<Output = Result<(), RoleError>> + Send;
    fn remove_member(
        &self,
        input: RemoveMemberInput,
    ) -> impl Future<Output = Result<(), RoleError>> + Send;
}

pub trait RoleService: Send + Sync {
    fn create(&self, input: CreateRoleInput) -> impl Future<Output = Result<(), RoleError>> + Send;
    fn delete(&self, input: DeleteRoleInput) -> impl Future<Output = Result<(), RoleError>> + Send;
    fn assign_member(
        &self,
        input: AssignMemberInput,
    ) -> impl Future<Output = Result<(), RoleError>> + Send;
    fn remove_member(
        &self,
        input: RemoveMemberInput,
    ) -> impl Future<Output = Result<(), RoleError>> + Send;
}
