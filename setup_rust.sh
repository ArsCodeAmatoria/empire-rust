#!/bin/bash

<!-- Header -->
# Empire-Rust Development Environment Setup Script
# This script sets up the Rust development environment for Empire-Rust

<!-- Error Handling -->
set -e  # Exit on error
set -u  # Exit on undefined variable

<!-- Colors -->
# Define colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

<!-- Functions -->
# Print status messages
print_status() {
    echo -e "${GREEN}[*] $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}[!] $1${NC}"
}

print_error() {
    echo -e "${RED}[x] $1${NC}"
}

<!-- Check Requirements -->
# Check if running as root
if [ "$EUID" -eq 0 ]; then
    print_warning "Please do not run this script as root"
    exit 1
fi

<!-- Install Rust -->
# Install Rust using rustup
print_status "Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

<!-- Source Environment -->
# Source the Rust environment
print_status "Sourcing Rust environment..."
source "$HOME/.cargo/env"

<!-- Install Components -->
# Install required Rust components
print_status "Installing Rust components..."
rustup component add rustfmt
rustup component add clippy
rustup component add rust-docs

<!-- Install Tools -->
# Install useful Rust tools
print_status "Installing Rust tools..."
cargo install cargo-edit
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-tarpaulin

<!-- Verify Installation -->
# Verify Rust installation
print_status "Verifying Rust installation..."
rustc --version
cargo --version

<!-- Create Project -->
# Create project directory if it doesn't exist
if [ ! -d "empire-rust" ]; then
    print_status "Creating project directory..."
    cargo new empire-rust
    cd empire-rust
else
    print_status "Project directory already exists"
    cd empire-rust
fi

<!-- Initialize Git -->
# Initialize git repository if not already initialized
if [ ! -d ".git" ]; then
    print_status "Initializing git repository..."
    git init
    git add .
    git commit -m "Initial commit"
fi

<!-- Create Configuration -->
# Create Rust configuration file
print_status "Creating Rust configuration..."
cat > .cargo/config.toml << EOF
[build]
rustflags = ["-D", "warnings"]

[profile.dev]
opt-level = 0
debug = true

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
EOF

<!-- Create Git Hooks -->
# Set up git hooks
print_status "Setting up git hooks..."
mkdir -p .git/hooks
cat > .git/hooks/pre-commit << EOF
#!/bin/bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
EOF
chmod +x .git/hooks/pre-commit

<!-- Install Dependencies -->
# Install project dependencies
print_status "Installing project dependencies..."
cargo add tokio --features full
cargo add serde --features derive
cargo add anyhow
cargo add thiserror
cargo add tracing
cargo add tracing-subscriber

<!-- Build Project -->
# Build the project
print_status "Building project..."
cargo build

<!-- Run Tests -->
# Run tests
print_status "Running tests..."
cargo test

<!-- Success Message -->
print_status "Setup completed successfully!"
print_status "You can now start developing Empire-Rust"
print_status "Run 'cargo run' to start the application" 