use authz_core::application::{AuthzRepositories, AuthzService};
pub mod pool;
#[derive(Clone)]
pub struct AppState {
    // Add shared state here
    pub service: AuthzService,
}

impl AppState {
    pub fn new(service: AuthzService) -> Self {
        Self { service }
    }
}

impl From<AuthzRepositories> for AppState {
    fn from(repositories: AuthzRepositories) -> Self {
        let service = AuthzService::new(repositories.server_repository);
        AppState { service }
    }
}
