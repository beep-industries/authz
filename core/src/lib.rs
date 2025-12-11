// Include the generated protobuf code
pub mod google {
    pub mod rpc {
        tonic::include_proto!("google.rpc");
    }
}

pub mod authzed {
    pub mod api {
        pub mod v1 {
            tonic::include_proto!("authzed.api.v1");
        }
    }
}

// Re-export commonly used types for convenience
pub use authzed::api::v1::{
    experimental_service_client::ExperimentalServiceClient,
    permissions_service_client::PermissionsServiceClient,
    schema_service_client::SchemaServiceClient, watch_service_client::WatchServiceClient,
};

pub mod infrastructure;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
