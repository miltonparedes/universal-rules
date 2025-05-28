The Universal Rules CLI (`urules`) converts universal Markdown-based rule definitions into agent-specific formats for tools like Cursor, Windsurf, and Claude.

## Quick Setup

```bash
git clone <repository-url>
cd universal-rules
./setup.sh
```

## Project Structure

```
src/
├── main.rs              # CLI entry point
├── converters/          # Agent-specific conversion logic
│   ├── cursor.rs       # Cursor converter
│   ├── windsurf.rs     # Windsurf converter
│   └── claude.rs       # Claude converter
├── rule_parser.rs      # Universal rule parsing
└── lib.rs              # Library exports

tests/                   # Integration tests
```

## Essential Commands

### Development Workflow
```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Build debug version
cargo build

# Build release version
cargo build --release

# Run the CLI
cargo run -- --help
cargo run -- --agent cursor --rules-dir ./example-rules
```

### Pre-commit Checks
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

## Testing

### Running Tests
```bash
cargo test                           # All tests
cargo test test_name                 # Specific test
cargo test -- --nocapture          # With output
cargo test --package rule_unifier_cli  # Package tests only
```

### Test Dependencies
- `assert_cmd`: CLI testing
- `predicates`: Output assertions
- `tempfile`: Temporary directories

## Key Components

### Rule Parser (`src/rule_parser.rs`)
Parses Markdown files with YAML frontmatter:
```yaml
---
description: "Rule description"
globs: ["*.rs", "*.ts"]
cursor_rule_type: "AutoAttached"
apply_globally: false
---
```

### Converters (`src/converters/`)
Implement `RuleConverter` trait to transform universal rules into agent-specific formats.

### Adding New Agent Support
1. Create `src/converters/new_agent.rs`
2. Implement `RuleConverter` trait
3. Add to `AgentName` enum in `main.rs`
4. Update converter instantiation in `main()`

## Common Tasks

### Debug Build and Test
```bash
cargo build
cargo test
cargo run -- --agent cursor
```

### Release Preparation
```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build --release
```

### Documentation
```bash
cargo doc --open              # Generate and open docs
cargo doc --document-private-items  # Include private items
```

### Dependency Management
```bash
cargo update                  # Update dependencies
cargo audit                   # Security audit (requires cargo-audit)
cargo tree                    # Dependency tree
```

## Error Handling Patterns

- Use `anyhow` for error propagation
- Provide descriptive error messages
- Handle file system operations safely
- Validate input parameters

## Code Quality Guidelines

- Run `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Add tests for new functionality
- Document public APIs with `///` comments
- Follow Rust naming conventions

## Debugging

```bash
# Debug build with symbols
cargo build

# Run with debug logging
RUST_LOG=debug cargo run -- --agent cursor

# Clean and rebuild
cargo clean && cargo build
```
