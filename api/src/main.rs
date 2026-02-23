use clap::Parser;
use lucid_api::{config::LucidApiConfig, server};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let config = LucidApiConfig::parse();

    let (router, api) = server::make(config.clone()).await;

    if config.dump_openapi {
        let json = api.to_pretty_json().unwrap();
        print!("{}", json);
    } else {
        if !config.dump_openapi {
            tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::try_from_default_env()
                        .unwrap_or("lucid_api=info,lucid_common=info,lucid_db=info".into()),
                )
                .pretty()
                .init();
        }

        let listener = TcpListener::bind(config.bind_addr)
            .await
            .expect("Failed to bind to address");

        info!("Listening on http://{:?}", config.bind_addr);

        axum::serve(listener, router)
            .await
            .expect("Failed to start server");
    }
}
