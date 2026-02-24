use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Serialize, Deserialize)]
struct Header {
    #[serde(rename = "jku")]
    pub jwks_url: String,
    #[serde(rename = "kid")]
    pub key_id: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    #[serde(rename = "iss")]
    pub issuer: String,
    #[serde(rename = "ak")]
    pub key_id: String,
}

#[derive(Subcommand)]
pub enum Command {
    Run,
    Register {
        /// The registration token provided by the Lucid API for agent registration.
        #[clap(long, short)]
        token: String,
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Command::Run => {
            println!("Starting Agent...");
        },
        Command::Register { token } => {
            println!("Registering Agent...");
            println!("Token: {}", token);
            let token_parts = token.split('.');
            let token_header = token_parts.clone().nth(0).expect("Failed to get token header part");
            let token_claims = token_parts.clone().nth(1).expect("Failed to get token claims part");
            println!("Token Claims B64: {}", token_claims);
            let token_header_decoded = BASE64_URL_SAFE_NO_PAD.decode(token_header).expect("Failed to decode token header");
            let token_claims_decoded = BASE64_URL_SAFE_NO_PAD.decode(token_claims).expect("Failed to decode token claims");
            let header: Header = serde_json::from_slice(&token_header_decoded).expect("Failed to parse token header");
            let claims: Claims = serde_json::from_slice(&token_claims_decoded).expect("Failed to parse token claims");
            println!("Token JWKS URI: {}", header.jwks_url);
            println!("Token JWKS Key ID: {}", header.key_id);
            println!("Token Claims Issuer: {}", claims.issuer);
            println!("Token Claims Key ID: {}", claims.key_id);
        },
    }
}
