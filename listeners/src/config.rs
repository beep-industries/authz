use clap::{Parser, command};

#[derive(Clone, Parser, Debug, Default)]
#[command(name = "communities-api")]
#[command(about = "Communities API Server", long_about = None)]
pub struct Config {
    #[arg(long, env)]
    pub authzed_url: String,
}
