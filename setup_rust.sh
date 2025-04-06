#!/bin/bash

# Add Rust to PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Source the Rust environment
source "$HOME/.cargo/env"

# Verify Rust installation
echo "Checking Rust installation..."
rustc --version
cargo --version

# Update Rust
rustup update

# Add necessary components
rustup component add rustfmt
rustup component add clippy
rustup component add rust-docs

echo "Rust environment setup complete." 