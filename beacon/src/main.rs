use std::{env, net::{IpAddr, SocketAddr}, str::FromStr};

use dropshot::ConfigDropshot;
use slog::Drain;
use tracing_slog::TracingSlogDrain;

use crate::context::Context;

mod api;
mod context;

#[tokio::main]
async fn main() {
    let mut args = env::args();
    let _ = args.next();
    let config_path = args.next();

    let config = lucid_beacon_config::BeaconConfig::new(
        config_path.map(|path| vec![path]),
    ).expect("Failed to load configuration");

    let ctx = Context::new(
        config.database.url,
        config.session,
    ).await.expect("building context");

    // Construct a shim to pipe dropshot logs into the global tracing logger
    let dropshot_logger = {
        let level_drain = slog::LevelFilter(TracingSlogDrain, slog::Level::Debug).fuse();
        let async_drain = slog_async::Async::new(level_drain).build().fuse();
        slog::Logger::root(async_drain, slog::o!())
    };

    let http_server = {
        dropshot::ServerBuilder::new(
            api::http_entrypoints::api(),
            ctx,
            dropshot_logger,
        )
            .config(ConfigDropshot {
                bind_address: SocketAddr::new(
                    IpAddr::from_str("0.0.0.0").expect("Failed to build bind address"),
                    8080,
                ),
                ..Default::default()
            })
            .start()
            .expect("starting server")
    };

    http_server.await.expect("Failed to run server");
}
