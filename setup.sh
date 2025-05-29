#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "=== Setting up Universal Rules Development Environment ==="

echo "=== Installing Rust (if not already installed) ==="
# Check if rustc is installed
if command -v rustc &> /dev/null
then
    echo "✓ Rust is already installed."
    rustc --version
else
    echo "Rust is not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # Add cargo to PATH for the current session
    source "$HOME/.cargo/env" 
    echo "✓ Rust installed successfully."
fi

echo ""
echo "=== Installing Rust development tools ==="
# Install useful development tools
echo "Installing rustfmt (code formatter)..."
rustup component add rustfmt

echo "Installing clippy (linter)..."
rustup component add clippy

echo "Installing rust-src (for IDE support)..."
rustup component add rust-src

echo ""
echo "=== Installing development dependencies ==="
echo "Fetching crates..."
cargo fetch
echo "Building project in debug mode..."
cargo build

echo ""
echo "=== Development environment setup complete! ==="
echo ""
echo "Available development commands:"
echo "  cargo build          - Build the project in debug mode"
echo "  cargo build --release - Build optimized release version"
echo "  cargo test           - Run all tests"
echo "  cargo test -- --nocapture - Run tests with output"
echo "  cargo clippy         - Run linter for code quality checks"
echo "  cargo fmt            - Format code automatically"
echo "  cargo run -- <args>  - Run the CLI tool with arguments"
echo "  cargo doc --open     - Generate and open documentation"
echo ""
echo "For VS Code users, consider installing the 'rust-analyzer' extension."
echo "For Vim/Neovim users, consider setting up rust-analyzer LSP."
echo ""
echo "Happy coding! 🦀"
