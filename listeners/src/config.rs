use authz_core::infrastructure::authzed::AuthZedConfig;
use clap::{Parser, command};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::lapin::RabbitClientConfig;

#[derive(Clone, Parser, Debug)]
#[command(name = "communities-api")]
#[command(about = "Communities API Server", long_about = None)]
pub struct Config {
    #[command(flatten)]
    pub rabbit_config: RabbitClientConfig,

    #[command(flatten)]
    pub authzed_config: AuthZedConfig,

    /// Path to the queue configuration JSON file
    #[arg(long, env = "QUEUE_CONFIG_PATH", default_value = "config/queues.json")]
    pub queue_config_path: PathBuf,

    #[clap(skip)]
    pub queue_config: Option<QueueConfig>,
}

impl Config {
    /// Load queue configuration from the JSON file and return updated Config
    pub fn with_queue_config(mut self) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(&self.queue_config_path)?;
        let config: QueueConfig = serde_json::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.queue_config = Some(config);
        Ok(self)
    }

    /// Get queue configuration, panics if not loaded
    pub fn queue_config(&self) -> &QueueConfig {
        self.queue_config
            .as_ref()
            .expect("Queue config not loaded. Call with_queue_config() first.")
    }
}

/// Queue configuration loaded from JSON file
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueueConfig {
    /// Server-related queue names
    pub server: ServerQueues,
    /// Channel-related queue names
    pub channel: ChannelQueues,
}

/// Server queue names
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerQueues {
    /// Queue name for create server operations
    pub create_server: String,
    /// Queue name for delete server operations
    pub delete_server: String,
}

/// Channel queue names
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelQueues {
    /// Queue name for create channel operations
    pub create_channel: String,
    /// Queue name for delete channel operations
    pub delete_channel: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_queue_config_success() {
        // Create a temporary JSON file
        let mut temp_file = NamedTempFile::new().unwrap();
        let json_content = r#"{
            "server": {
                "create_server": "test_create_server_queue",
                "delete_server": "test_delete_server_queue"
            },
            "channel": {
                "create_channel": "test_create_channel_queue",
                "delete_channel": "test_delete_channel_queue"
            }
        }"#;
        temp_file.write_all(json_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Create a minimal config with just the queue_config_path
        let config = Config {
            rabbit_config: RabbitClientConfig {
                uri: "amqp://localhost:5672".to_string(),
                consumer_tag_suffix: "test".to_string(),
            },
            authzed_config: AuthZedConfig {
                endpoint: "localhost:50051".to_string(),
                token: Some("test_token".to_string()),
            },
            queue_config_path: temp_file.path().to_path_buf(),
            queue_config: None,
        };

        // Load the queue config
        let config = config.with_queue_config().unwrap();

        // Verify the loaded config
        assert_eq!(
            config.queue_config().server.create_server,
            "test_create_server_queue"
        );
        assert_eq!(
            config.queue_config().server.delete_server,
            "test_delete_server_queue"
        );
        assert_eq!(
            config.queue_config().channel.create_channel,
            "test_create_channel_queue"
        );
        assert_eq!(
            config.queue_config().channel.delete_channel,
            "test_delete_channel_queue"
        );
    }

    #[test]
    fn test_load_queue_config_file_not_found() {
        let config = Config {
            rabbit_config: RabbitClientConfig {
                uri: "amqp://localhost:5672".to_string(),
                consumer_tag_suffix: "test".to_string(),
            },
            authzed_config: AuthZedConfig {
                endpoint: "localhost:50051".to_string(),
                token: Some("test_token".to_string()),
            },
            queue_config_path: PathBuf::from("nonexistent_file.json"),
            queue_config: None,
        };

        let result = config.with_queue_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_queue_config_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let invalid_json = r#"{ invalid json }"#;
        temp_file.write_all(invalid_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = Config {
            rabbit_config: RabbitClientConfig {
                uri: "amqp://localhost:5672".to_string(),
                consumer_tag_suffix: "test".to_string(),
            },
            authzed_config: AuthZedConfig {
                endpoint: "localhost:50051".to_string(),
                token: Some("test_token".to_string()),
            },
            queue_config_path: temp_file.path().to_path_buf(),
            queue_config: None,
        };

        let result = config.with_queue_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_queue_config() {
        let json = r#"{
            "server": {
                "create_server": "my_queue",
                "delete_server": "my_delete_queue"
            },
            "channel": {
                "create_channel": "my_create_channel_queue",
                "delete_channel": "my_delete_channel_queue"
            }
        }"#;

        let config: QueueConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.server.create_server, "my_queue");
        assert_eq!(config.server.delete_server, "my_delete_queue");
        assert_eq!(config.channel.create_channel, "my_create_channel_queue");
        assert_eq!(config.channel.delete_channel, "my_delete_channel_queue");
    }
}
