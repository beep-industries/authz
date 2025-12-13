use authz_core::application::{AuthzRepositories, AuthzService};

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
