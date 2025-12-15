use crate::{
    config::RoleQueues,
    rabbit::{
        consumers::{AppState, pool::Consumers},
        role::handler::{assign_member_to_role, delete_role, remove_member_from_role, upsert_role},
    },
};

pub fn role_consumers(queue_config: &RoleQueues) -> Consumers<AppState> {
    Consumers::new()
        .add(&queue_config.upsert_role, upsert_role)
        .add(&queue_config.delete_role, delete_role)
        .add(&queue_config.member_assigned_to_role, assign_member_to_role)
        .add(
            &queue_config.member_removed_from_role,
            remove_member_from_role,
        )
}
