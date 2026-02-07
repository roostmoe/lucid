use std::net::{SocketAddr, SocketAddrV4};

use anyhow::Result;
use tap::TapFallible;
use tracing::level_filters::LevelFilter;
use tracing_logfmt::{EventsFormatter, FieldsFormatter};
use tracing_subscriber::{
    fmt::{
        format::{DefaultFields, Format},
        SubscriberBuilder
    },
    EnvFilter,
};

use crate::config::ServerLogLevel;
use crate::{config::{AppConfig, ServerLogFormat}, context::LucidContext, server::{ServerConfig, server}};

mod config;
mod context;
mod error;
mod server;
mod endpoints;

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut args = std::env::args();
    let _ = args.next();
    let config_path = args.next();

    let config = AppConfig::new(config_path.map(|path| vec![path]))?;

    match config.log_format {
        ServerLogFormat::Json => fmt_subscriber(config.log_level).json().init(),
        ServerLogFormat::Pretty => fmt_subscriber(config.log_level).pretty().init(),
        ServerLogFormat::Logfmt => logfmt_subscriber(config.log_level).init(),
    }

    tracing::info!("Initialized logger");

    let ctx = LucidContext::new("".to_string()).await?;

    let config = ServerConfig {
        context: ctx,
        server_address: SocketAddr::V4(SocketAddrV4::new(
            "0.0.0.0"
                .parse()
                .tap_err(|err| tracing::error!(?err, "Failed to parse server address"))?,
            config.server_port,
        ))
    };

    let server = server(config)
        .tap_err(|err| {
            tracing::error!(?err, "Failed to construct server");
        })
        .expect("Failed to start server")
        .start();

    server?
        .await
        .tap_err(|err| tracing::error!(?err, "Server exited with an error"))
        .expect("Failed to start server");

    tracing::error!("Server completed without an error");

    Ok(())
}

fn fmt_subscriber(level: ServerLogLevel) -> SubscriberBuilder<DefaultFields, Format, EnvFilter> {
    tracing_subscriber::fmt()
        .with_file(false)
        .with_line_number(false)
        .with_env_filter(EnvFilter::builder()
            .with_default_directive(LevelFilter::from(level).into())
            .from_env_lossy()
        )
}

fn logfmt_subscriber(level: ServerLogLevel) -> SubscriberBuilder<FieldsFormatter, EventsFormatter, EnvFilter> {
    tracing_logfmt::builder()
        .with_location(false)
        .subscriber_builder()
        .with_env_filter(EnvFilter::builder()
            .with_default_directive(LevelFilter::from(level).into())
            .from_env_lossy()
        )
}
