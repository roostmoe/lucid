use clap::{Parser, Subcommand};
use lucid_db::storage::mongodb::MongoDBStorage;

use crate::commands::CreateUserParams;

mod commands;

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,

    #[clap(
        short = 'D',
        long,
        env = "LUCID_API_DB_URL",
        default_value = "mongodb://localhost:27017/lucid"
    )]
    db_url: String,
}

#[derive(Clone, Subcommand)]
pub enum Command {
    #[command(name = "create-user")]
    CreateUser(CreateUserParams),
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let stg = MongoDBStorage::new(&args.db_url)
        .await
        .expect("Failed to connect to MongoDB");

    match args.command {
        Command::CreateUser(params) => {
            commands::create_user(&stg, params)
                .await
                .expect("Failed to create user");
        }
    }
}
