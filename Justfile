# Universal Rules - Development Commands
# Run `just --list` to see all available commands

# Default recipe - shows help
default:
    @just --list

# Development commands
# ===================

# Install development dependencies and setup environment
setup:
    ./setup.sh

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode (optimized)
build-release:
    cargo build --release

# Build with all features enabled
build-all-features:
    cargo build --all-features

# Run all tests
test:
    cargo test

# Run tests with output visible
test-verbose:
    cargo test -- --nocapture

# Run tests and show coverage (requires cargo-tarpaulin)
test-coverage:
    cargo tarpaulin --out Html

# Run specific test by name
test-name TEST_NAME:
    cargo test {{TEST_NAME}} -- --nocapture

# Code quality commands
# ====================

# Format code using rustfmt
fmt:
    cargo fmt

# Check if code is formatted correctly
fmt-check:
    cargo fmt -- --check

# Run clippy linter
lint:
    cargo clippy

# Run clippy with all features and strict settings
lint-strict:
    cargo clippy --all-features --all-targets -- -D warnings

# Fix automatically fixable clippy warnings
lint-fix:
    cargo clippy --fix --allow-dirty --allow-staged

# Run all quality checks (format + lint + test)
check:
    @echo "🔍 Running format check..."
    just fmt-check
    @echo "🔍 Running lint check..."
    just lint-strict
    @echo "🔍 Running tests..."
    just test
    @echo "✅ All checks passed!"

# Documentation commands
# =====================

# Generate and open documentation
docs:
    cargo doc --open

# Generate documentation without opening
docs-build:
    cargo doc

# Generate documentation with private items
docs-private:
    cargo doc --document-private-items --open

# Run commands
# ============

# Run the CLI tool with arguments
run *ARGS:
    cargo run -- {{ARGS}}

# Run the release version with arguments
run-release *ARGS:
    cargo run --release -- {{ARGS}}

# Run with example arguments (help command)
run-help:
    cargo run -- --help

# Installation commands
# ====================

# Install the binary locally using cargo
install:
    cargo install --path .

# Install the binary to a custom location
install-to PATH:
    cargo install --path . --root {{PATH}}

# Uninstall the locally installed binary
uninstall:
    cargo uninstall rule_unifier_cli

# Maintenance commands
# ===================

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Audit dependencies for security vulnerabilities
audit:
    cargo audit

# Show dependency tree
deps:
    cargo tree

# Utility commands
# ===============

# Show project information
info:
    @echo "Project: rule_unifier_cli"
    @echo "Version: $(cargo pkgid | cut -d# -f2)"
    @echo "Rust version: $(rustc --version)"
    @echo "Cargo version: $(cargo --version)"

# Benchmark the application (requires criterion)
bench:
    cargo bench

# Profile the application (requires cargo-profiler)
profile:
    cargo build --release
    @echo "Run your profiling tools on: target/release/urules"

# Development workflow commands
# ============================

# Quick development cycle: format, lint, test
dev:
    @echo "🔧 Formatting code..."
    just fmt
    @echo "🔍 Running lint..."
    just lint
    @echo "🧪 Running tests..."
    just test
    @echo "✅ Development cycle complete!"

# Prepare for commit: run all checks and cleanup
pre-commit:
    @echo "🚀 Preparing for commit..."
    just fmt
    just lint-strict
    just test
    @echo "✅ Ready to commit!"

# Full rebuild from scratch
rebuild:
    just clean
    just build

# Release preparation workflow
release-prep:
    @echo "🎯 Preparing release..."
    just clean
    just fmt-check
    just lint-strict
    just test
    just build-release
    just docs-build
    @echo "✅ Release ready!"

# CI/CD simulation
ci:
    @echo "🤖 Running CI checks..."
    just fmt-check
    just lint-strict
    just test
    just build-release
    @echo "✅ CI checks passed!"

# Watch mode (requires cargo-watch)
# ================================

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Watch for changes and run checks
watch-check:
    cargo watch -x check

# Watch for changes and run clippy
watch-lint:
    cargo watch -x clippy

# Watch for changes and run the application
watch-run *ARGS:
    cargo watch -x 'run -- {{ARGS}}'

# Example usage commands
# =====================

# Show example usage of the CLI tool
examples:
    @echo "Example usage:"
    @echo "  just run --help                    # Show help"
    @echo "  just run-release input.yaml        # Process a file"
    @echo "  just run --version                 # Show version"
    @echo ""
    @echo "Development examples:"
    @echo "  just dev                           # Quick dev cycle"
    @echo "  just watch-test                    # Watch mode testing"
    @echo "  just pre-commit                    # Pre-commit checks"

# Install development tools
install-tools:
    @echo "Installing development tools..."
    cargo install cargo-watch
    cargo install cargo-tarpaulin
    cargo install cargo-outdated
    cargo install cargo-audit
    @echo "✅ Development tools installed!"