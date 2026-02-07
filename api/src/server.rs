use std::{error::Error, net::SocketAddr};

use dropshot::{ApiDescription, ConfigDropshot, ServerBuilder};
use slog::Drain;
use tracing_slog::TracingSlogDrain;

use crate::{context::LucidContext, endpoints::auth::sign_up};

pub struct ServerConfig {
    pub context: LucidContext,
    pub server_address: SocketAddr,
}

pub fn server(config: ServerConfig) -> Result<ServerBuilder<LucidContext>, Box<dyn Error + Send + Sync>> {
    let config_dropshot = ConfigDropshot {
        bind_address: config.server_address,
        default_request_body_max_bytes: 1024 * 1024,
        ..Default::default()
    };

    // Construct a shim to pipe dropshot logs into the global tracing logger
    let dropshot_logger = {
        let level_drain = slog::LevelFilter(TracingSlogDrain, slog::Level::Debug).fuse();
        let async_drain = slog_async::Async::new(level_drain).build().fuse();
        slog::Logger::root(async_drain, slog::o!())
    };

    let mut api = ApiDescription::new();

    api.register(sign_up)?;

    Ok(ServerBuilder::new(api, config.context, dropshot_logger).config(config_dropshot))
}
