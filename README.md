# Empire-Rust

A post-exploitation framework written in Rust, designed for security professionals and penetration testers.

## Features

- Modern, asynchronous architecture
- Cross-platform support
- Secure communication protocol
- Modular design for easy extension
- Beautiful CLI interface with ASCII art

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/empire-rust.git
cd empire-rust

# Build the project
cargo build --release

# Install the binary
cargo install --path .
```

## Usage

### Starting the Server

```bash
empire server --host 0.0.0.0 --port 1337
```

### Starting an Agent

```bash
empire agent --host 127.0.0.1 --port 1337 --username admin --password secret
```

### Listing Connected Agents

```bash
empire list
```

### Executing Commands

```bash
empire exec --agent-id <agent-id> "whoami"
```

### CLI Options

- `--verbose`: Enable verbose output
- `--no-color`: Disable colored output
- `--help`: Show help information
- `--version`: Show version information

## Project Structure

```
empire-rust/
├── src/
│   ├── cli/         # Command-line interface
│   ├── core/        # Core functionality
│   ├── server/      # Server implementation
│   └── client/      # Client implementation
├── docs/            # Documentation
├── tests/           # Test files
└── scripts/         # Helper scripts
```

## Documentation

- [Architecture Overview](docs/architecture/overview.md)
- [API Documentation](docs/api/README.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the original Empire framework
- Built with Rust for performance and safety
- Uses modern async/await patterns 