use clap::{Parser, Subcommand};

mod config;
mod crypto;
mod register;

#[derive(Parser)]
#[command(name = "lucid-agent")]
pub struct Args {
    #[command(subcommand)]
    command: Command,
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

    match args.command {
        Command::Run => {
            println!("Starting Agent...");
            // Future: run the agent daemon
            Ok(())
        }
        Command::Register { token } => register::register(&token).await,
        Command::Unregister => unregister(),
    }
}

fn unregister() -> anyhow::Result<()> {
    use crate::config::{auth_cert_path, auth_key_path, ca_cert_path};

    let mut removed = false;

    for path in [auth_key_path(), auth_cert_path(), ca_cert_path()] {
        if path.exists() {
            std::fs::remove_file(&path)?;
            println!("Removed: {}", path.display());
            removed = true;
        }
    }

    if removed {
        println!("✓ Local credentials removed");
        println!("  Note: The agent is still registered on the server.");
        println!("  An admin must revoke it via the API.");
    } else {
        println!("No credentials found - agent was not registered.");
    }

    Ok(())
}
