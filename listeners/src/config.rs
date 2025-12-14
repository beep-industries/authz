use authz_core::infrastructure::authzed::AuthZedConfig;
use clap::{Parser, command};

use crate::lapin::RabbitClientConfig;

#[derive(Clone, Parser, Debug, Default)]
#[command(name = "communities-api")]
#[command(about = "Communities API Server", long_about = None)]
pub struct Config {
    #[command(flatten)]
    pub rabbit_config: RabbitClientConfig,

    #[command(flatten)]
    pub authzed_config: AuthZedConfig,
}
