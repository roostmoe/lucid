use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::AgentConfig;

mod config;
mod util;

mod client;
mod commands;
mod plugins;

#[derive(Parser)]
#[command(name = "lucid-agent")]
pub struct Args {
    #[command(subcommand)]
    command: Command,

    #[arg(
        long,
        short,
        global = true,
        env = "LUCID_AGENT_CONFIG_PATH",
        default_value = "/etc/lucid/agent.toml"
    )]
    config_path: PathBuf,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the agent daemon
    Run,

    /// Register this agent with the Lucid API
    Register {
        /// Activation key JWT from the Lucid console
        #[arg(long, short)]
        token: String,
    },

    /// Remove local registration credentials
    Unregister,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = AgentConfig::from_file(args.config_path)
        .expect("Failed to load config - ensure the config file exists and is valid TOML");

    match args.command {
        Command::Run => commands::run::run(config).await,
        Command::Register { token } => commands::registration::register(&token, config).await,
        Command::Unregister => commands::registration::unregister(config),
    }
}
