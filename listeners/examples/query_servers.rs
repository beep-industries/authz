//! Example for querying and displaying server relationships from SpiceDB
//!
//! This example shows:
//! - How to read relationships from SpiceDB
//! - Display servers and their owners
//! - Filter by resource type, owner, or server
//!
//! Run with:
//! ```
//! cargo run --example query_servers
//! cargo run --example query_servers -- --owner-id user_123
//! cargo run --example query_servers -- --server-id server_456
//! cargo run --example query_servers -- --all
//! ```

use authz_core::{
    authzed::api::v1::{RelationshipFilter, SubjectFilter},
    infrastructure::authzed::{AuthZedClient, AuthZedConfig},
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "query_servers")]
#[command(about = "Query server relationships from SpiceDB", long_about = None)]
struct Args {
    /// Filter by owner ID (e.g., user_123)
    #[arg(long)]
    owner_id: Option<String>,

    /// Filter by server ID (e.g., server_456)
    #[arg(long)]
    server_id: Option<String>,

    /// Show all server relationships
    #[arg(long, default_value = "false")]
    all: bool,

    /// SpiceDB endpoint
    #[arg(long, env = "AUTHZED_ENDPOINT", default_value = "localhost:50051")]
    authzed_endpoint: String,

    /// SpiceDB authentication token
    #[arg(long, env = "AUTHZED_TOKEN", default_value = "foobar")]
    authzed_token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🔍 Querying SpiceDB at {}", args.authzed_endpoint);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Connect to SpiceDB
    let config = AuthZedConfig {
        endpoint: args.authzed_endpoint.clone(),
        token: Some(args.authzed_token.clone()),
    };
    let client = AuthZedClient::new(config).await?;

    // Build the relationship filter
    let filter = build_filter(&args);

    println!("\n📊 Filter Configuration:");
    print_filter_info(&args);
    println!();

    // Query relationships
    let relationships = client.read_relationships(filter).await?;

    if relationships.is_empty() {
        println!("❌ No relationships found matching the filter.");
        println!("\n💡 Try:");
        println!("  - Sending a message first: ./listeners/examples/send_create_server.sh");
        println!("  - Using --all to see all relationships");
        println!("  - Checking if the listeners service is running");
        return Ok(());
    }

    println!("✅ Found {} relationship(s)\n", relationships.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Display relationships in a nice format
    for (idx, rel) in relationships.iter().enumerate() {
        let resource = rel.resource.as_ref().unwrap();
        let subject = rel.subject.as_ref().unwrap();

        println!("\n#{} Relationship:", idx + 1);
        println!("  Resource Type: {}", resource.object_type);
        println!("  Resource ID:   {}", resource.object_id);
        println!("  Relation:      {}", rel.relation);
        println!(
            "  Subject Type:  {}",
            subject.object.as_ref().unwrap().object_type
        );
        println!(
            "  Subject ID:    {}",
            subject.object.as_ref().unwrap().object_id
        );

        // Pretty print what this means
        let resource_type = &resource.object_type;
        let resource_id = &resource.object_id;
        let relation = &rel.relation;
        let subject_type = &subject.object.as_ref().unwrap().object_type;
        let subject_id = &subject.object.as_ref().unwrap().object_id;

        println!("\n  📝 Meaning:");
        println!(
            "     {} '{}' is the {} of {} '{}'",
            subject_type, subject_id, relation, resource_type, resource_id
        );
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n✨ Summary:");
    println!("  Total relationships: {}", relationships.len());

    // Group by type for summary
    let servers: Vec<_> = relationships
        .iter()
        .filter(|r| r.resource.as_ref().unwrap().object_type == "server")
        .collect();

    if !servers.is_empty() {
        println!("  Servers found: {}", servers.len());

        // List unique owners
        let owners: std::collections::HashSet<String> = servers
            .iter()
            .map(|r| {
                r.subject
                    .as_ref()
                    .unwrap()
                    .object
                    .as_ref()
                    .unwrap()
                    .object_id
                    .clone()
            })
            .collect();

        println!("  Unique owners: {}", owners.len());
    }

    println!();
    Ok(())
}

fn build_filter(args: &Args) -> RelationshipFilter {
    if args.all {
        // Query all server relationships
        return RelationshipFilter {
            resource_type: "server".to_string(),
            ..Default::default()
        };
    }

    if let Some(server_id) = &args.server_id {
        // Filter by specific server
        return RelationshipFilter {
            resource_type: "server".to_string(),
            optional_resource_id: server_id.clone(),
            ..Default::default()
        };
    }

    if let Some(owner_id) = &args.owner_id {
        // Filter by owner (subject)
        return RelationshipFilter {
            resource_type: "server".to_string(),
            optional_subject_filter: Some(SubjectFilter {
                subject_type: "user".to_string(),
                optional_subject_id: owner_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
    }

    // Default: show all servers
    RelationshipFilter {
        resource_type: "server".to_string(),
        ..Default::default()
    }
}

fn print_filter_info(args: &Args) {
    if args.all {
        println!("  🔍 Showing: All server relationships");
    } else if let Some(server_id) = &args.server_id {
        println!("  🔍 Showing: Relationships for server '{}'", server_id);
    } else if let Some(owner_id) = &args.owner_id {
        println!("  🔍 Showing: Servers owned by user '{}'", owner_id);
    } else {
        println!("  🔍 Showing: All server relationships (default)");
    }
}
