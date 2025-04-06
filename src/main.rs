//! Empire-Rust - A post-exploitation framework written in Rust
//!
//! This is the main entry point for the Empire-Rust framework.

mod cli;
mod core;
mod server;
mod client;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::init()?;
    Ok(())
} 