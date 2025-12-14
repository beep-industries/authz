use std::convert::Infallible;
use std::future::Future;

use clap::Parser;
use lapin::{Channel, Connection, Consumer, options::BasicConsumeOptions, types::FieldTable};
use prost::Message;
use thiserror::Error;
use tokio_stream::StreamExt;

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
    pub async fn new(config: RabbitClientConfig) -> Result<Self, RabbitClientError> {
        let connection = Connection::connect(&config.uri, lapin::ConnectionProperties::default())
            .await
            .map_err(|e| RabbitClientError::StartupError { msg: e.to_string() })?;
        let channel = connection
            .create_channel()
            .await
            .map_err(|e| RabbitClientError::StartupError { msg: e.to_string() })?;
        Ok(RabbitClient {
            connection,
            consumer_tag_suffix: config.consumer_tag_suffix,
            channel,
        })
    }

    pub async fn shutdown(&self) -> Result<(), RabbitClientError> {
        self.connection
            .close(0, "Shutdown")
            .await
            .map_err(|e| RabbitClientError::StartupError { msg: e.to_string() })
    }

    async fn create_consumer(&self, queue_name: String) -> Result<Consumer, RabbitClientError> {
        self.channel
            .basic_consume(
                queue_name.as_str(),
                format!("{}-{}", queue_name, self.consumer_tag_suffix).as_str(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| RabbitClientError::StartupError { msg: e.to_string() })
    }

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
        let mut consumer = self.create_consumer(queue_name.clone()).await?;

        while let Some(message) = consumer.next().await {
            let lapin_delivery = match message {
                Ok(delivery) => delivery,
                Err(_) => {
                    // TODO: log error
                    // Failed to extract message
                    continue;
                }
            };
            let content = match M::decode(&lapin_delivery.data[..]) {
                Ok(content) => content,
                Err(_) => {
                    // TODO: log error
                    // Failed to decode message
                    continue;
                }
            };
            let process_result = handler.handle(state.clone(), content).await;
            if process_result.is_ok() {
                match lapin_delivery
                    .ack(lapin::options::BasicAckOptions::default())
                    .await
                {
                    Ok(_) => {
                        continue;
                    }
                    Err(_) => {
                        // log error
                        // Failed to ack message
                        continue;
                    }
                };
            } else {
                // log info
                // Handler returned error, not acknowledging message
                continue;
            }
        }
        Ok(())
    }
}
