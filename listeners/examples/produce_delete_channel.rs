//! Example producer for sending ChannelDeleted messages to RabbitMQ
//!
//! Run with:
//! ```
//! cargo run --example produce_delete_channel
//! cargo run --example produce_delete_channel -- --channel-id channel_123
//! ```

use clap::Parser;
use events_protobuf::communities_events::ChannelDeleted;
use lapin::{
    BasicProperties, Connection, ConnectionProperties,
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
};
use prost::Message;

#[derive(Parser, Debug)]
#[command(name = "produce_delete_channel")]
#[command(about = "Send ChannelDeleted messages to RabbitMQ", long_about = None)]
struct Args {
    /// Channel ID to delete
    #[arg(long, default_value = "channel_789")]
    channel_id: String,

    /// RabbitMQ URI
    #[arg(
        long,
        env = "RABBIT_URI",
        default_value = "amqp://guest:guest@localhost:5672"
    )]
    rabbit_uri: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Connecting to RabbitMQ at {}", args.rabbit_uri);
    let conn = Connection::connect(&args.rabbit_uri, ConnectionProperties::default()).await?;

    let channel = conn.create_channel().await?;

    // Declare the queue (idempotent operation)
    let queue_name = "channel.delete_channel";
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    println!("Queue '{}' declared successfully", queue_name);

    // Create the protobuf message
    let delete_channel_msg = ChannelDeleted {
        channel_id: args.channel_id.clone(),
    };

    println!(
        "Created message: channel_id='{}'",
        delete_channel_msg.channel_id
    );

    // Serialize the protobuf message
    let mut buf = Vec::new();
    delete_channel_msg.encode(&mut buf)?;

    println!("Serialized message to {} bytes", buf.len());

    // Publish the message
    channel
        .basic_publish(
            "",
            queue_name,
            BasicPublishOptions::default(),
            &buf,
            BasicProperties::default()
                .with_content_type("application/x-protobuf".into())
                .with_delivery_mode(2), // persistent
        )
        .await?;

    println!(
        "✅ Message published successfully to queue '{}'",
        queue_name
    );

    // Close the connection
    conn.close(0, "Normal shutdown").await?;

    Ok(())
}
