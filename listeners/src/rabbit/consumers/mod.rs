use authz_core::application::{AuthzRepositories, AuthzService};
use authz_core::domain::common::service::Service;
use authz_core::domain::server::port::ServerRepository;

pub mod pool;

#[derive(Clone)]
pub struct AppState<
    S = authz_core::infrastructure::server::repository::authzed::AuthzedServerRepository,
> where
    S: ServerRepository,
{
    // Add shared state here
    pub service: Service<S>,
}

impl<S> AppState<S>
where
    S: ServerRepository,
{
    pub fn new(service: Service<S>) -> Self {
        Self { service }
    }
}

impl From<AuthzRepositories> for AppState {
    fn from(repositories: AuthzRepositories) -> Self {
        let service = AuthzService::new(repositories.server_repository);
        AppState { service }
    }
}
