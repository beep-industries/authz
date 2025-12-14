use std::convert::Infallible;
use std::future::Future;

use clap::Parser;
use lapin::{Channel, Connection, Consumer, options::BasicConsumeOptions, types::FieldTable};
use prost::Message;
use thiserror::Error;
use tokio_stream::StreamExt;
use tracing::{debug, error, info, instrument, warn};

pub struct RabbitClient {
    connection: Connection,
    channel: Channel,
    consumer_tag_suffix: String,
}

#[derive(Clone, Parser, Debug, Default)]
pub struct RabbitClientConfig {
    #[arg(long = "rabbit-uri", env = "RABBIT_URI", default_value = "localhost")]
    pub uri: String,
    #[arg(
        long = "rabbit-consumer-tag-suffix",
        env = "RABBIT_CONSUMER_TAG_SUFFIX",
        default_value = "default"
    )]
    pub consumer_tag_suffix: String,
}

pub type QueueName = String;

pub trait MessageHandler<S, M>: Send + Sync {
    type Future: Future<Output = Result<(), Infallible>> + Send;
    fn handle(&self, state: S, message: M) -> Self::Future;
}

impl<S, M, F, Fut> MessageHandler<S, M> for F
where
    F: Fn(S, M) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), Infallible>> + Send,
{
    type Future = Fut;
    fn handle(&self, state: S, message: M) -> Self::Future {
        self(state, message)
    }
}

impl<S, M, H> MessageHandler<S, M> for std::sync::Arc<H>
where
    H: MessageHandler<S, M> + ?Sized,
    S: Clone,
{
    type Future = H::Future;
    fn handle(&self, state: S, message: M) -> Self::Future {
        (**self).handle(state, message)
    }
}

#[derive(Debug, Error)]
pub enum RabbitClientError {
    #[error("Service could not start: {msg}")]
    StartupError { msg: String },
}

impl RabbitClient {
    #[instrument(skip_all, fields(uri = %config.uri, consumer_tag_suffix = %config.consumer_tag_suffix))]
    pub async fn new(config: RabbitClientConfig) -> Result<Self, RabbitClientError> {
        info!("Connecting to RabbitMQ");
        let connection = Connection::connect(&config.uri, lapin::ConnectionProperties::default())
            .await
            .map_err(|e| {
                error!("Failed to connect to RabbitMQ: {}", e);
                RabbitClientError::StartupError { msg: e.to_string() }
            })?;
        info!("RabbitMQ connection established");

        debug!("Creating RabbitMQ channel");
        let channel = connection.create_channel().await.map_err(|e| {
            error!("Failed to create RabbitMQ channel: {}", e);
            RabbitClientError::StartupError { msg: e.to_string() }
        })?;
        info!("RabbitMQ channel created successfully");

        Ok(RabbitClient {
            connection,
            consumer_tag_suffix: config.consumer_tag_suffix,
            channel,
        })
    }

    #[instrument(skip_all)]
    pub async fn shutdown(&self) -> Result<(), RabbitClientError> {
        info!("Shutting down RabbitMQ connection");
        self.connection.close(0, "Shutdown").await.map_err(|e| {
            error!("Failed to shutdown RabbitMQ connection: {}", e);
            RabbitClientError::StartupError { msg: e.to_string() }
        })?;
        info!("RabbitMQ connection closed");
        Ok(())
    }

    #[instrument(skip(self), fields(queue_name = %queue_name))]
    async fn create_consumer(&self, queue_name: String) -> Result<Consumer, RabbitClientError> {
        let consumer_tag = format!("{}-{}", queue_name, self.consumer_tag_suffix);
        debug!(consumer_tag = %consumer_tag, "Creating consumer for queue");

        let consumer = self
            .channel
            .basic_consume(
                queue_name.as_str(),
                consumer_tag.as_str(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| {
                error!("Failed to create consumer for queue {}: {}", queue_name, e);
                RabbitClientError::StartupError { msg: e.to_string() }
            })?;

        info!(queue_name = %queue_name, consumer_tag = %consumer_tag, "Consumer created successfully");
        Ok(consumer)
    }

    #[instrument(skip(self, state, handler), fields(queue_name = %queue_name))]
    pub async fn consume_messages<S, M, H>(
        &self,
        state: S,
        queue_name: String,
        handler: H,
    ) -> Result<(), RabbitClientError>
    where
        M: Message + Default + 'static,
        S: Clone + Send + Sync + 'static,
        H: MessageHandler<S, M>,
    {
        info!(queue_name = %queue_name, "Starting message consumption");
        let mut consumer = self.create_consumer(queue_name.clone()).await?;
        let mut message_count = 0u64;

        while let Some(message) = consumer.next().await {
            message_count += 1;
            debug!(queue_name = %queue_name, message_count, "Received message");

            let lapin_delivery = match message {
                Ok(delivery) => delivery,
                Err(e) => {
                    error!(queue_name = %queue_name, error = %e, "Failed to extract message from queue");
                    continue;
                }
            };

            let content = match M::decode(&lapin_delivery.data[..]) {
                Ok(content) => content,
                Err(e) => {
                    error!(
                        queue_name = %queue_name,
                        error = %e,
                        delivery_tag = lapin_delivery.delivery_tag,
                        "Failed to decode message"
                    );
                    continue;
                }
            };

            debug!(queue_name = %queue_name, delivery_tag = lapin_delivery.delivery_tag, "Message decoded successfully");

            let process_result = handler.handle(state.clone(), content).await;
            if process_result.is_ok() {
                match lapin_delivery
                    .ack(lapin::options::BasicAckOptions::default())
                    .await
                {
                    Ok(_) => {
                        debug!(
                            queue_name = %queue_name,
                            delivery_tag = lapin_delivery.delivery_tag,
                            "Message acknowledged successfully"
                        );
                        continue;
                    }
                    Err(e) => {
                        error!(
                            queue_name = %queue_name,
                            delivery_tag = lapin_delivery.delivery_tag,
                            error = %e,
                            "Failed to acknowledge message"
                        );
                        continue;
                    }
                };
            } else {
                warn!(
                    queue_name = %queue_name,
                    delivery_tag = lapin_delivery.delivery_tag,
                    "Handler returned error, not acknowledging message"
                );
                continue;
            }
        }

        warn!(queue_name = %queue_name, total_messages = message_count, "Consumer stream ended");
        Ok(())
    }
}
