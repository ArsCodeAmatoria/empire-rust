//! Empire-Rust - A post-exploitation framework written in Rust
//!
//! This is the main entry point for the Empire-Rust framework.
//! The framework provides a command-line interface for managing
//! agents and executing commands across multiple systems.

mod cli;
mod core;
mod server;
mod client;

use anyhow::Result;

/// Main entry point for the Empire-Rust framework.
///
/// This function:
/// 1. Initializes the command-line interface
/// 2. Parses command-line arguments
/// 3. Executes the requested command
///
/// # Returns
///
/// * `Ok(())` if the program executed successfully
/// * `Err(anyhow::Error)` if an error occurred
#[tokio::main]
async fn main() -> Result<()> {
    cli::init()?;
    Ok(())
} 