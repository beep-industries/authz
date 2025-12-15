use crate::{
    config::PermissionOverrideQueues,
    rabbit::{
        consumers::{AppState, pool::Consumers},
        permission_override::handler::{delete_permission_override, upsert_permission_override},
    },
};

pub fn permission_override_consumers(
    queue_config: &PermissionOverrideQueues,
) -> Consumers<AppState> {
    Consumers::new()
        .add(
            &queue_config.upsert_permission_override,
            upsert_permission_override,
        )
        .add(
            &queue_config.delete_permission_override,
            delete_permission_override,
        )
}
