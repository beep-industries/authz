//! Example producer for sending ChannelCreated messages to RabbitMQ
//!
//! Run with:
//! ```
//! cargo run --example produce_create_channel
//! cargo run --example produce_create_channel -- --channel-id channel_123 --server-id server_456
//! ```

use clap::Parser;
use events_protobuf::communities_events::ChannelCreated;
use lapin::{
    BasicProperties, Connection, ConnectionProperties,
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
};
use prost::Message;

#[derive(Parser, Debug)]
#[command(name = "produce_create_channel")]
#[command(about = "Send ChannelCreated messages to RabbitMQ", long_about = None)]
struct Args {
    /// Channel ID
    #[arg(long, default_value = "channel_789")]
    channel_id: String,

    /// Server ID that owns the channel
    #[arg(long, default_value = "server_456")]
    server_id: String,

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
    let queue_name = "channel.create_channel";
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
    let create_channel_msg = ChannelCreated {
        channel_id: args.channel_id.clone(),
        server_id: args.server_id.clone(),
    };

    println!(
        "Created message: channel_id='{}', server_id='{}'",
        create_channel_msg.channel_id, create_channel_msg.server_id
    );

    // Serialize the protobuf message
    let mut buf = Vec::new();
    create_channel_msg.encode(&mut buf)?;

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
