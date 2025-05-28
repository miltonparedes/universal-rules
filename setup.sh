#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "=== Installing Rust (if not already installed) ==="
# Check if rustc is installed
if command -v rustc &> /dev/null
then
    echo "Rust is already installed."
else
    echo "Rust is not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # Add cargo to PATH for the current session
    source "$HOME/.cargo/env" 
fi

echo "=== Building the project ==="
cargo build --release

echo ""
echo "=== Build complete ==="
echo "The binary is located at target/release/rule_unifier_cli"
echo ""
echo "To make the 'rule_unifier_cli' command available system-wide,"
echo "you can copy it to a directory in your system's PATH."
echo "For example:"
echo "  sudo cp target/release/rule_unifier_cli /usr/local/bin/urules"
echo "or for a user-local installation (ensure ~/.local/bin is in your PATH):"
echo "  mkdir -p ~/.local/bin"
echo "  cp target/release/rule_unifier_cli ~/.local/bin/urules"
echo ""
echo "Alternatively, you can install the binary directly using cargo from the project root:"
echo "  cargo install --path ."
echo "This will install it to ~/.cargo/bin/ (ensure this directory is in your PATH)."
echo "The binary will be named 'rule_unifier_cli'."

echo ""
echo "Setup complete. You might need to restart your terminal or source your shell profile (e.g., source ~/.bashrc or source ~/.zshrc) for PATH changes to take effect."
