# Empire-Rust

A post-exploitation framework written in Rust, inspired by the original Empire project.

## Overview

Empire-Rust is a modern implementation of a post-exploitation framework, providing a robust and efficient platform for security testing and penetration testing. Built with Rust for performance and safety, it offers a command-line interface for both server and client operations.

## Features

- Asynchronous server-client architecture
- Command execution and task management
- Agent management system
- Secure communication protocol
- Cross-platform support

## Installation

1. Ensure you have Rust and Cargo installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Clone the repository:
```bash
git clone https://github.com/yourusername/empire-rust.git
cd empire-rust
```

3. Build the project:
```bash
cargo build --release
```

## Usage

### Starting the Server

```bash
cargo run -- server --host 0.0.0.0 --port 1337
```

### Connecting as a Client

```bash
cargo run -- client --host 127.0.0.1 --port 1337
```

## Project Structure

- `src/core/` - Core data structures and traits
- `src/server/` - Server implementation
- `src/client/` - Client implementation
- `src/main.rs` - CLI interface and main program logic

## Dependencies

- tokio - Async runtime
- serde - Serialization
- clap - Command-line interface
- hyper - HTTP server
- chrono - Date/time handling
- uuid - Unique identifier generation

## Security Considerations

This tool is intended for authorized security testing and penetration testing purposes only. Unauthorized use of this tool against systems you do not own or have explicit permission to test is illegal.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 