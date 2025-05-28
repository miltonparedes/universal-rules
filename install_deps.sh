#!/usr/bin/env bash
set -e

# Installs Rust toolchain and dependencies for the project
# Run this script after cloning the repository

if ! command -v rustup >/dev/null 2>&1; then
    echo "rustup not found, installing..."
    curl https://sh.rustup.rs -sSf | sh -s -- -y
    export PATH="$HOME/.cargo/bin:$PATH"
fi

rustup toolchain install stable
rustup default stable

# Fetch project dependencies
cargo fetch

echo "Dependencies installed"
