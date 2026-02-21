use clap::Parser;
use std::net::SocketAddr;

#[derive(Clone, Debug, Parser)]
pub struct LucidApiConfig {
    #[clap(short, long, env = "LUCID_API_BIND_ADDR", default_value = "0.0.0.0:4000")]
    pub bind_addr: SocketAddr,

    #[clap(long, env = "LUCID_API_PUBLIC_URL", default_value = "http://localhost:4000")]
    pub public_url: String,
}
