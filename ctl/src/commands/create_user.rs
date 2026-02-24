use clap::Parser;
use lucid_common::{caller::Caller, params::CreateLocalUserParams};
use lucid_db::storage::UserStore;

#[derive(Clone, Parser)]
pub struct CreateUserParams {
    #[clap(short, long)]
    pub display_name: String,

    #[clap(short, long)]
    pub email: String,

    #[clap(short, long)]
    pub password: String,
}

pub async fn create_user(
    stg: &impl UserStore,
    CreateUserParams {
        display_name,
        email,
        password,
    }: CreateUserParams,
) -> anyhow::Result<()> {
    let new_user = UserStore::create_local(
        stg,
        Caller::System,
        CreateLocalUserParams {
            display_name,
            email,
            password,
        },
    )
    .await?;

    println!("Created user with ID {}", new_user.id.unwrap());

    Ok(())
}
