//! Command-line interface module for the Empire-Rust framework.
//!
//! This module provides a user-friendly command-line interface for interacting
//! with the Empire framework. It includes commands for starting the server,
//! connecting agents, and executing commands.

use clap::{Parser, Subcommand};
use log::{error, info};
use std::process::exit;
use colored::*;

/// ASCII art banner for the Empire framework
const BANNER: &str = r#"
    ███████╗███╗   ███╗██████╗ ██╗██████╗ ███████╗
    ██╔════╝████╗ ████║██╔══██╗██║██╔══██╗██╔════╝
    █████╗  ██╔████╔██║██████╔╝██║██████╔╝█████╗  
    ██╔══╝  ██║╚██╔╝██║██╔═══╝ ██║██╔══██╗██╔══╝  
    ███████╗██║ ╚═╝ ██║██║     ██║██║  ██║███████╗
    ╚══════╝╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝
    ════════════════════════════════════════════════
    ██████╗ ██╗   ██╗███████╗████████╗
    ██╔══██╗██║   ██║██╔════╝╚══██╔══╝
    ██████╔╝██║   ██║███████╗   ██║   
    ██╔══██╗██║   ██║╚════██║   ██║   
    ██║  ██║╚██████╔╝███████║   ██║   
    ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   
"#;

/// Main CLI structure for the Empire framework.
///
/// This struct defines the top-level command-line interface, including
/// global options and subcommands.
#[derive(Parser)]
#[command(name = "empire")]
#[command(version = "0.1.0")]
#[command(about = "A post-exploitation framework written in Rust", long_about = None)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Disable colored output
    #[arg(short, long, global = true)]
    no_color: bool,
}

/// Available commands for the Empire CLI.
///
/// Each variant represents a different operation that can be performed
/// through the command-line interface.
#[derive(Subcommand)]
enum Commands {
    /// Start the Empire server
    Server {
        /// Host address to bind to
        #[arg(short, long, default_value = "0.0.0.0")]
        host: String,

        /// Port to listen on
        #[arg(short, long, default_value = "1337")]
        port: u16,
    },

    /// Start an Empire agent
    Agent {
        /// Server host to connect to
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,

        /// Server port to connect to
        #[arg(short, long, default_value = "1337")]
        port: u16,

        /// Username for authentication
        #[arg(short, long)]
        username: String,

        /// Password for authentication
        #[arg(short, long)]
        password: String,
    },

    /// List connected agents
    List,

    /// Execute a command on an agent
    Exec {
        /// Agent ID to execute command on
        #[arg(short, long)]
        agent_id: String,

        /// Command to execute
        command: String,
    },
}

impl Cli {
    /// Display the Empire banner and version information.
    ///
    /// This method prints a colorful ASCII art banner along with version
    /// information and a description of the framework.
    pub fn display_banner() {
        if !Cli::parse().no_color {
            colored::control::set_override(true);
        }

        let banner = BANNER;
        let version = "v0.1.0".yellow();
        let description = "A post-exploitation framework written in Rust".cyan();
        let separator = "════════════════════════════════════════════════════════════════════════════════".blue();
        
        println!("\n{}", banner);
        println!("{}", separator);
        println!("Empire-Rust {} - {}", version, description);
        println!("{}\n", separator);
    }

    /// Parse and execute the CLI commands.
    ///
    /// This method processes the command-line arguments and executes
    /// the appropriate action based on the selected subcommand.
    ///
    /// # Arguments
    ///
    /// * `self` - The CLI instance containing the parsed arguments
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the command executed successfully
    /// * `Err(Box<dyn Error>)` if an error occurred
    pub async fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Commands::Server { host, port } => {
                info!("Starting Empire server on {}:{}", host, port);
                // TODO: Start server
                Ok(())
            }
            Commands::Agent { host, port, username, password } => {
                info!("Starting Empire agent connecting to {}:{}", host, port);
                // TODO: Start agent
                Ok(())
            }
            Commands::List => {
                info!("Listing connected agents");
                // TODO: List agents
                Ok(())
            }
            Commands::Exec { agent_id, command } => {
                info!("Executing command on agent {}: {}", agent_id, command);
                // TODO: Execute command
                Ok(())
            }
        }
    }
}

/// Initialize the CLI interface.
///
/// This function sets up the command-line interface, displays the banner,
/// and processes the command-line arguments.
///
/// # Returns
///
/// * `Ok(())` if initialization was successful
/// * `Err(Box<dyn Error>)` if an error occurred
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    Cli::display_banner();
    
    let cli = Cli::parse();
    
    // Set up logging
    let log_level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    
    env_logger::Builder::new()
        .filter_level(log_level)
        .init();

    // Execute the command
    if let Err(e) = cli.execute().await {
        error!("Error: {}", e);
        exit(1);
    }

    Ok(())
} 