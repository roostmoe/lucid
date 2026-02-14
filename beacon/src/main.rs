use std::{convert::Infallible, env, sync::Arc};

use dropshot::ConfigDropshot;
use lucid_beacon_config::{LogFormat, LogLevel, LoggingConfig};
use slog::{Drain, SendSyncRefUnwindSafeDrain};
use tracing::level_filters::LevelFilter;
use tracing_slog::TracingSlogDrain;
use tracing_subscriber::{layer::SubscriberExt, registry, util::SubscriberInitExt};

use crate::context::Context;

mod api;
mod app;
mod context;

#[tokio::main]
async fn main() {
    let mut args = env::args();
    let _ = args.next();
    let config_path = args.next();

    let config = lucid_beacon_config::BeaconConfig::new(config_path.map(|path| vec![path]))
        .expect("Failed to load configuration");

    let dropshot_logger = setup_tracing(config.clone().logging);

    let ctx = Context::new(config.clone())
        .await
        .expect("building context");

    let http_server = {
        dropshot::ServerBuilder::new(api::http_entrypoints::api(), ctx, dropshot_logger)
            .config(ConfigDropshot {
                bind_address: config.server.bind_addr,
                ..Default::default()
            })
            .start()
            .expect("starting server")
    };

    http_server.await.expect("Failed to run server");
}

type DropshotLogger = slog::Logger<Arc<dyn SendSyncRefUnwindSafeDrain<Ok = (), Err = Infallible>>>;

fn setup_tracing(cfg: LoggingConfig) -> DropshotLogger {
    let layer = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_file(false);

    let level_filter = match cfg.level {
        LogLevel::Trace => LevelFilter::TRACE,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Error => LevelFilter::ERROR,
    };

    match cfg.format {
        LogFormat::Json => registry().with(layer.json()).with(level_filter).init(),
        LogFormat::Pretty => registry().with(layer.pretty()).with(level_filter).init(),
    }

    let logger = {
        let level_drain = slog::LevelFilter(TracingSlogDrain, slog::Level::Debug).fuse();
        let async_drain = slog_async::Async::new(level_drain).build().fuse();
        slog::Logger::root(async_drain, slog::o!())
    };

    logger
}
