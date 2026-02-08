use std::{error::Error, net::SocketAddr};

use dropshot::{ApiDescription, ConfigDropshot, ServerBuilder};
use slog::Drain;
use tracing_slog::TracingSlogDrain;

use crate::{
    context::Context,
    endpoints::{health_check, login},
};

pub struct ServerConfig {
    pub context: Context,
    pub server_address: SocketAddr,
}

pub fn server(
    config: ServerConfig,
) -> Result<ServerBuilder<Context>, Box<dyn Error + Send + Sync>> {
    let config_dropshot = ConfigDropshot {
        bind_address: config.server_address,
        default_request_body_max_bytes: 1024 * 1024, // 1 MiB
        ..Default::default()
    };

    let dropshot_logger = {
        let level_drain = slog::LevelFilter(TracingSlogDrain, slog::Level::Debug).fuse();
        let async_drain = slog_async::Async::new(level_drain).build().fuse();
        slog::Logger::root(async_drain, slog::o!())
    };

    let mut api = ApiDescription::new();

    // Endpoints
    api.register(health_check)
        .expect("Failed to register endpoint");
    api.register(login::login)
        .expect("Failed to register endpoint");
    api.register(login::login_session)
        .expect("Failed to register endpoint");
    api.register(login::logout)
        .expect("Failed to register endpoint");
    api.register(login::whoami)
        .expect("Failed to register endpoint");

    Ok(ServerBuilder::new(api, config.context, dropshot_logger).config(config_dropshot))
}
