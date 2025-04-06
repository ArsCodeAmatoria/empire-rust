mod core;
mod server;
mod client;

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{error, info};
use std::path::PathBuf;

use crate::client::Client;
use crate::core::EmpireError;
use crate::server::Server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Empire server
    Server {
        /// Host to bind to
        #[arg(short, long, default_value = "0.0.0.0")]
        host: String,
        
        /// Port to listen on
        #[arg(short, long, default_value = "1337")]
        port: u16,
    },
    
    /// Connect to an Empire server
    Client {
        /// Server host
        #[arg(short, long)]
        host: String,
        
        /// Server port
        #[arg(short, long, default_value = "1337")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Server { host, port } => {
            info!("Starting Empire server on {}:{}", host, port);
            let server = Server::new();
            server.start().await.map_err(|e| {
                error!("Server error: {}", e);
                anyhow::anyhow!(e)
            })?;
        }
        Commands::Client { host, port } => {
            info!("Connecting to Empire server at {}:{}", host, port);
            let client = Client::new(host, port).map_err(|e| {
                error!("Client error: {}", e);
                anyhow::anyhow!(e)
            })?;
            client.connect().await.map_err(|e| {
                error!("Connection error: {}", e);
                anyhow::anyhow!(e)
            })?;
        }
    }
    
    Ok(())
} 