use std::{net::SocketAddr, str::FromStr};

use tracing_subscriber::EnvFilter;

use crate::{
    config::LucidConfig,
    context::Context,
    server::{ServerConfig, server},
};

mod config;
mod context;
mod endpoints;
mod server;
mod session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let _ = args.next();
    let config_path = args.next();

    let config = LucidConfig::new(config_path.map(|p| vec![p]))?;

    tracing_subscriber::fmt()
        .with_file(false)
        .with_line_number(false)
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let ctx = Context::new(
        config.database_url,
        config.mode,
        config.session,
    )
        .await
        .expect("building context");

    let listen_addr = SocketAddr::from_str(&config.bind_address)
        .expect("invalid server bind address");

    tracing::info!(%listen_addr, "starting API server");

    let server = server(ServerConfig {
        context: ctx,
        server_address: listen_addr,
    })
    .expect("failed to construct server")
    .start()?;

    server.await.expect("failed to start server");

    tracing::error!("server stopped unexpectedly");

    Ok(())
}
