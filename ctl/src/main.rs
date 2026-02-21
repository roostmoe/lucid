use clap::{Parser, Subcommand};
use lucid_common::params::CreateLocalUserParams;
use lucid_db::storage::{UserStore, mongodb::MongoDBStorage};

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,

    #[clap(short = 'D', long, env = "LUCID_API_DB_URL", default_value = "mongodb://localhost:27017/lucid")]
    db_url: String,
}

#[derive(Clone, Subcommand)]
pub enum Command {
    #[command(name = "create-user")]
    CreateUser {
        #[clap(short, long)]
        display_name: String,

        #[clap(short, long)]
        email: String,

        #[clap(short, long)]
        password: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let stg = MongoDBStorage::new(&args.db_url)
        .await
        .expect("Failed to connect to MongoDB");

    match args.command {
        Command::CreateUser { display_name, email, password } => {
            println!("Creating user with email: {} and password: {}", email, password);
            let new_user = UserStore::create_local(&stg, CreateLocalUserParams {
                display_name,
                email,
                password,
            })
                .await
                .expect("Failed to create user");

            println!("Created user with ID {}", new_user.id.unwrap());
        }
    }
}
