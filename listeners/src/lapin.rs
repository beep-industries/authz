use lapin::{Channel, Connection, Consumer, options::BasicConsumeOptions, types::FieldTable};
use prost::Message;
use thiserror::Error;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

use crate::rabbit::consumers::AppState;

pub struct RabbitClient {
    connection: Connection,
    channel: Channel,
    consumer_tag_suffix: String,
}

pub struct RabbitClientConfig {
    pub uri: String,
    pub consumer_tag_suffix: String,
}

pub type QueueName = String;

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

    pub async fn consume_messages<M, E>(
        &self,
        app_state: AppState,
        queue_name: String,
        handler: fn(AppState, M) -> Result<(), E>,
    ) -> Result<JoinHandle<QueueName>, RabbitClientError>
    where
        M: Message + Default + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut consumer = self.create_consumer(queue_name).await?;
        let consumer_thread_handler = tokio::spawn(async move {
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
                let process_result = handler(app_state.clone(), content);
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
            consumer.queue().to_string()
        });
        Ok(consumer_thread_handler)
    }
}
