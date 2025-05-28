# Universal Rule Unifier CLI (`urules`)

## Purpose

The Universal Rule Unifier CLI (`urules`) is a tool designed to simplify and centralize the management of coding assistance rules for various AI-powered coding agents. It allows you to define rules in a universal Markdown-based format and then convert them into the specific formats required by different agents like Cursor, Windsurf, and Claude.

This approach helps maintain consistency across different tools and makes it easier to manage and version control your custom instructions and prompts.

## Installation

You can install `urules` either from source using the setup script or directly using `cargo install`.

### From Source (Recommended)

1.  **Ensure you have the source code.** If you've cloned the repository, you're ready.
2.  **Run the setup script:**
    ```bash
    ./setup.sh
    ```
    This script will:
    *   Check for Rust and install it if necessary.
    *   Build the project in release mode (`cargo build --release`).
    *   Provide instructions on how to copy the compiled binary (`target/release/rule_unifier_cli`) to a directory in your system's PATH, for example, as `urules`. Follow the instructions provided by the script.

### Using `cargo install`

If you have the source code, you can also install the binary directly using `cargo install` from the project's root directory:

```bash
cargo install --path .
```

This command will compile the binary and install it into `~/.cargo/bin/`. By default, the binary will be named `rule_unifier_cli`.
*   If you want to use `urules` as the command name, you can rename the binary after installation (e.g., `mv ~/.cargo/bin/rule_unifier_cli ~/.cargo/bin/urules`).
*   Alternatively, you can create an alias in your shell's configuration file (e.g., `alias urules='rule_unifier_cli'`).
*   The `setup.sh` script also provides guidance on this.

## Usage (`urules`)

The basic command structure for the tool is:

```bash
urules --agent <AGENT_NAME> [OPTIONS]
```
*(Note: If you've installed the binary as `rule_unifier_cli` and haven't renamed or aliased it, use `rule_unifier_cli` in the commands below.)*

### Main Options

*   `-r, --rules-dir <PATH>`: Specifies the directory containing your universal rule files (Markdown `.md` files).
    *   Default: `.rules` (relative to the current directory where you run the command).
*   `-a, --agent <AGENT_NAME>`: **Required.** Specifies the target agent for which to generate rules.
    *   Available agents:
        *   `cursor`: For Cursor.ai specific rules.
        *   `windsurf`: For Windsurf (e.g., global and workspace rules).
        *   `claude`: For a single concatenated Claude prompt file.
*   `-o, --output-dir <PATH>`: Specifies the directory where the agent-specific rules will be generated.
    *   Default: `.` (the current directory where you run the command).
*   `--no-gitignore` (Upcoming): This flag will be used to prevent the tool from automatically creating or updating a `.gitignore` file in the output directory (useful if you want to commit the generated rules).

### Examples

These examples assume you are running the command from the root of your project where your `.rules` directory (or equivalent) is located.

*   **Generate rules for Cursor (using default directories):**
    ```bash
    urules --agent cursor
    ```
    This will look for rules in `./.rules/` and output Cursor rules to `./.cursor/rules/`.

*   **Generate rules for Windsurf in a specific project's output directory:**
    ```bash
    urules --agent windsurf --output-dir ./project_x_config
    ```
    This will look for rules in `./.rules/` and output Windsurf rules to `./project_x_config/global_rules.md` and `./project_x_config/.windsurf/rules/`.

*   **Generate a CLAUDE.md file from a custom rules directory:**
    ```bash
    urules --agent claude --rules-dir ./my_claude_rules --output-dir ./claude_prompts
    ```
    This will look for rules in `./my_claude_rules/` and output `CLAUDE.md` to `./claude_prompts/CLAUDE.md`.

## Universal Rule Structure

Universal rules are defined as Markdown (`.md`) files located within the directory specified by `--rules-dir`. Each file represents a single rule.

### YAML Frontmatter

Each rule file can optionally start with a YAML frontmatter block, enclosed by `---` lines. The following fields are supported:

*   `description: String` (Optional): A brief description of what the rule does. This is used for comments in some generated rule formats.
*   `globs: Vec<String>` (Optional): A list of glob patterns (e.g., `["*.rs", "src/utils/*.ts"]`) that determine which files this rule applies to. This is primarily used by agents like Cursor for auto-attaching rules or by Windsurf for workspace rule targeting.
*   `apply_globally: bool` (Optional, defaults to `false`): If `true`, the rule is considered a "global" rule. This is primarily used by the Windsurf converter to place the rule content into `global_rules.md`. For other agents, this flag might influence default behavior if not overridden by other settings.
*   `cursor_rule_type: String` (Optional): Specifies the type of rule for Cursor. This directly influences how the rule is formatted for Cursor.
    *   Examples:
        *   `"Always"`: Rule is always active (maps to `alwaysApply: true` for Cursor).
        *   `"AutoAttached"`: Rule is attached based on `globs` (default behavior if globs are present and not "Always" or "AgentRequested").
        *   `"AgentRequested"`: Rule is available for the agent to request (maps to `agentRequested: true` for Cursor).
        *   `"Manual"`: Rule is manually invokable (default if no specific type or relevant frontmatter is provided).

### Example Universal Rule File

Filename: `my_rust_best_practices.md`
Located in: `<rules-dir>/my_rust_best_practices.md` (e.g., `.rules/my_rust_best_practices.md`)

```markdown
---
description: Enforces Rust best practices for error handling and logging.
globs: ["*.rs"]
cursor_rule_type: "AutoAttached"
apply_globally: false
---

## Error Handling Guidance

When writing Rust code, ensure that you handle all `Result` types appropriately. Use `?` for propagating errors within functions that return `Result`. For errors that should terminate the program or be handled at a higher level, consider using `expect()` with a descriptive message or proper error logging.

Avoid using `unwrap()` on `Option` or `Result` types in production code unless you can absolutely guarantee that the value is present.

## Logging

Use the `log` crate for logging. Prefer structured logging if possible.
Example: `info!("User {} logged in", user_id);`
```

## Extending the Tool

`urules` is designed to be extensible. To add support for a new coding agent, you need to:

1.  Implement the `RuleConverter` trait (defined in `src/converters/mod.rs`).
2.  Add the new agent to the `AgentName` enum in `src/main.rs`.
3.  Update the `match` statement in `main()` to instantiate your new converter.

## License

This project is licensed under the MIT License. (Or Apache 2.0, etc. - Placeholder)
