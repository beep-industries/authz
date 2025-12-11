use std::env;
use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Get the workspace root (parent of core/)
    let workspace_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .to_path_buf();

    let proto_dir = workspace_root.join("proto");

    // Configure tonic-build to compile the AuthZed protobuf files
    tonic_build::configure()
        .build_server(false) // We only need the client
        .build_client(true)
        .compile_well_known_types(true) // Include google.protobuf types
        .extern_path(".google.protobuf", "::prost_types") // Map google.protobuf to prost_types
        .compile_protos(
            &[
                // Google RPC types
                proto_dir.join("google/rpc/status.proto"),
                // Core API proto files from AuthZed
                proto_dir.join("authzed/api/v1/permission_service.proto"),
                proto_dir.join("authzed/api/v1/schema_service.proto"),
                proto_dir.join("authzed/api/v1/watch_service.proto"),
                proto_dir.join("authzed/api/v1/experimental_service.proto"),
            ],
            &[&proto_dir], // Include directory
        )?;

    // Rerun if proto files change
    println!("cargo:rerun-if-changed={}", proto_dir.display());

    Ok(())
}
