use clap::{Parser, Subcommand};
use lucid_api::{
    auth::encrypted_ca::{EncryptedCa, generate_ca},
    config::LucidApiConfig,
    server,
};
use lucid_db::storage::mongodb::MongoDBStorage;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "lucid-api")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    config: LucidApiConfig,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new Certificate Authority
    GenerateCa {
        /// Overwrite existing CA (DANGER: invalidates all agent certs)
        #[arg(long)]
        force: bool,
    },
    /// Run the API server (default)
    Serve,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::GenerateCa { force }) => {
            // Initialize minimal logging for CLI
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::new("info"))
                .init();

            if let Err(e) = run_generate_ca(&cli.config, force).await {
                eprintln!("Error generating CA: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Serve) | None => {
            run_server(cli.config).await;
        }
    }
}

async fn run_generate_ca(config: &LucidApiConfig, force: bool) -> anyhow::Result<()> {
    // Load encryption key
    let encryption_key = EncryptedCa::encryption_key_from_env()
        .map_err(|e| anyhow::anyhow!("Failed to load encryption key: {}", e))?;

    // Connect to MongoDB
    let db = MongoDBStorage::new(&config.mongodb_uri).await?;

    // Generate CA
    info!("Generating CA certificate...");
    let ca_info = generate_ca(&db, &encryption_key, force)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to generate CA: {}", e))?;

    println!("\n✅ CA certificate generated successfully!\n");
    println!("Fingerprint: {}", ca_info.fingerprint);
    println!("Issued:      {}", ca_info.issued_at);
    println!("Expires:     {}", ca_info.expires_at);
    println!(
        "\nAgents can fetch the CA certificate from: {}/.well-known/lucid/agent",
        config.public_url
    );

    Ok(())
}

async fn run_server(config: LucidApiConfig) {
    let (router, api) = server::make(config.clone()).await;

    if config.dump_openapi {
        let json = api.to_pretty_json().unwrap();
        print!("{}", json);
        return;
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or("lucid_api=info,lucid_common=info,lucid_db=info".into()),
        )
        .pretty()
        .init();

    if config.tls.enabled {
        run_tls_server(config, router).await;
    } else {
        run_plain_server(config, router).await;
    }
}

async fn run_plain_server(config: LucidApiConfig, router: axum::Router) {
    let listener = TcpListener::bind(config.bind_addr)
        .await
        .expect("Failed to bind to address");

    info!("Listening on http://{}", config.bind_addr);

    axum::serve(listener, router)
        .await
        .expect("Failed to start server");
}

async fn run_tls_server(config: LucidApiConfig, router: axum::Router) {
    use axum_server::tls_rustls::RustlsConfig;
    use std::sync::Arc;

    let cert_path = config
        .tls
        .cert_path
        .as_ref()
        .expect("TLS enabled but LUCID_API_TLS_CERT not set");
    let key_path = config
        .tls
        .key_path
        .as_ref()
        .expect("TLS enabled but LUCID_API_TLS_KEY not set");

    // Load rustls config
    let tls_config = if let Some(ca_cert_path) = &config.tls.ca_cert_path {
        // mTLS mode - verify client certificates
        info!("Configuring mTLS with CA cert from {:?}", ca_cert_path);

        let ca_cert = std::fs::read(ca_cert_path).expect("Failed to read CA certificate");
        let server_cert = std::fs::read(cert_path).expect("Failed to read server certificate");
        let server_key = std::fs::read(key_path).expect("Failed to read server key");

        // Parse CA cert for client verification
        let ca_certs: Vec<_> = rustls_pemfile::certs(&mut ca_cert.as_slice())
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to parse CA certificate");

        let mut root_store = rustls::RootCertStore::empty();
        for cert in ca_certs {
            root_store
                .add(cert)
                .expect("Failed to add CA to root store");
        }

        // Parse server cert chain
        let server_certs: Vec<_> = rustls_pemfile::certs(&mut server_cert.as_slice())
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to parse server certificate");

        // Parse server key
        let server_key = rustls_pemfile::private_key(&mut server_key.as_slice())
            .expect("Failed to parse server key")
            .expect("No private key found in file");

        // Build client verifier that requests certs
        let client_verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(root_store))
            .build()
            .expect("Failed to build client verifier");

        let rustls_config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(server_certs, server_key)
            .expect("Failed to build TLS config");

        RustlsConfig::from_config(Arc::new(rustls_config))
    } else {
        // TLS only, no client cert verification
        RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .expect("Failed to load TLS configuration")
    };

    info!("Listening on https://{}", config.bind_addr);

    axum_server::bind_rustls(config.bind_addr, tls_config)
        .serve(router.into_make_service())
        .await
        .expect("Failed to start TLS server");
}
