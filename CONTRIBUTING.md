# Contributing to Empire-Rust

Thank you for your interest in contributing to Empire-Rust! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. Please be respectful and considerate of others.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/empire-rust.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Push to your fork: `git push origin feature/your-feature-name`
6. Create a Pull Request

## Development Guidelines

### Code Style

- Follow the Rust style guide
- Use `rustfmt` to format your code
- Run `cargo clippy` to check for common mistakes
- Document all public APIs using Rust's documentation system

### Testing

- Write unit tests for new functionality
- Ensure all tests pass before submitting a PR
- Add integration tests for complex features
- Document test coverage in PR descriptions

### Documentation

- Update the README.md if necessary
- Document new features in the appropriate module
- Add examples for complex functionality
- Keep the CHANGELOG.md up to date

### Security

- Report security vulnerabilities privately to the maintainers
- Follow secure coding practices
- Document any security considerations in your PR

## Pull Request Process

1. Ensure your PR description clearly describes the problem and solution
2. Include relevant test cases
3. Update documentation as needed
4. Ensure all CI checks pass
5. Request review from maintainers

## Development Environment Setup

1. Install Rust and Cargo
2. Install development tools:
   ```bash
   rustup component add rustfmt
   rustup component add clippy
   ```
3. Configure git hooks (optional):
   ```bash
   cargo install cargo-husky
   cargo husky install
   ```

## Project Structure

- `src/core/` - Core data structures and traits
- `src/server/` - Server implementation
- `src/client/` - Client implementation
- `src/main.rs` - CLI interface and main program logic
- `tests/` - Test files
- `docs/` - Documentation
- `examples/` - Usage examples

## Questions and Support

If you have questions or need support:
- Open an issue on GitHub
- Join our community chat (if available)
- Contact the maintainers directly

Thank you for contributing to Empire-Rust! 