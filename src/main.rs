use std::path::PathBuf;
use clap::{Parser, ValueEnum};
use anyhow::Result;

pub mod universal_rule;
pub mod rule_parser;
pub mod converters; // New module for all converters
pub mod gitignore_manager;

use crate::rule_parser::discover_and_parse_rules;
// Import the trait and specific converter structs
use crate::converters::RuleConverter;
use crate::converters::cursor::CursorConverter;
use crate::converters::windsurf::WindsurfConverter;
use crate::converters::claude::ClaudeConverter;
use crate::gitignore_manager::update_gitignore; // Import the new function

#[derive(ValueEnum, Clone, Debug, PartialEq)]
/// Specifies the target AI agent for rule generation.
pub enum AgentName { // Made AgentName public
    /// Rules for Cursor.ai.
    Cursor,
    /// Rules for Windsurf.
    Windsurf,
    /// Rules for Claude (concatenated into a single file).
    Claude,
}

// No changes needed for Display impl
impl std::fmt::Display for AgentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentName::Cursor => write!(f, "Cursor"),
            AgentName::Windsurf => write!(f, "Windsurf"),
            AgentName::Claude => write!(f, "Claude"),
        }
    }
}


/// Command-line interface for the Universal Rule Unifier.
/// This tool processes universal rule files (Markdown with optional YAML frontmatter)
/// and converts them into formats specific to different AI coding agents.
#[derive(Parser, Debug)]
#[clap(name = "urules", version = "0.1.0", about = "Unifies coding agent rules from a universal format.", long_about = None)]
struct Cli {
    /// Directory containing the universal rule files (Markdown `.md` files).
    #[clap(short, long, value_parser, default_value = ".rules", help = "Directory containing universal rule files (.md).")]
    rules_dir: PathBuf,

    /// Target AI agent for which to generate rules.
    #[clap(short, long, value_enum, help = "Target agent for rule generation.")]
    agent: AgentName,

    /// Directory where the agent-specific rules will be generated.
    #[clap(short, long, value_parser, default_value = ".", help = "Directory to output generated agent-specific rules.")]
    output_dir: PathBuf,

    /// Disable automatic update of .gitignore in the output directory.
    #[clap(long, default_value_t = false, help = "Disable automatic update of .gitignore.")]
    no_gitignore: bool,
}

/// Main entry point for the CLI application.
///
/// Orchestrates the process of:
/// 1. Parsing command-line arguments.
/// 2. Validating the existence of the rules directory.
/// 3. Discovering and parsing universal rules from the specified directory.
/// 4. Validating the existence of the output directory, creating it if necessary.
/// 5. Selecting the appropriate rule converter based on the chosen agent.
/// 6. Generating agent-specific rules using the selected converter.
/// 7. Optionally updating the .gitignore file in the output directory.
/// 8. Printing a success message with the output location.
fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Ensure the specified rules directory exists
    if !cli.rules_dir.exists() {
        eprintln!("Error: Rules directory {:?} does not exist.", cli.rules_dir);
        eprintln!("Please create it or specify a valid directory with --rules-dir.");
        std::process::exit(1); // Exit with an error code
    }

    // Discover and parse all universal rules from the rules directory
    let rules = discover_and_parse_rules(&cli.rules_dir)
        .map_err(|e| {
            // Provide context for errors during rule discovery and parsing
            eprintln!("Error discovering or parsing rules from {:?}: {}", cli.rules_dir, e);
            e
        })?;

    // If no rules are found, inform the user and exit gracefully
    if rules.is_empty() {
        println!("No rules found in {:?}.", cli.rules_dir);
        return Ok(());
    }

    // Ensure the output directory exists, create it if it doesn't
    if !cli.output_dir.exists() {
        std::fs::create_dir_all(&cli.output_dir)
            .map_err(|e| {
                // Provide context for errors during output directory creation
                eprintln!("Error creating output directory {:?}: {}", cli.output_dir, e);
                e
            })?;
    }

    // Select the appropriate converter based on the agent specified via CLI
    let converter: Box<dyn RuleConverter> = match cli.agent {
        AgentName::Cursor => Box::new(CursorConverter),
        AgentName::Windsurf => Box::new(WindsurfConverter),
        AgentName::Claude => Box::new(ClaudeConverter),
    };

    // Generate the agent-specific rules using the selected converter
    converter.generate_rules(&rules, &cli.output_dir)?;
    
    // Update .gitignore if not disabled by the user
    if !cli.no_gitignore {
        if let Err(e) = update_gitignore(&cli.output_dir, &cli.agent) {
            // Log the error but don't cause the program to fail, as .gitignore update is auxiliary
            eprintln!("Warning: Failed to update .gitignore in {:?}: {}", cli.output_dir, e);
        }
    }

    // Print a success message, including a description of where the rules were generated
    println!(
        "Rules generated successfully for {} in {}",
        cli.agent, // Uses the Display impl of AgentName
        converter.get_output_description(&cli.output_dir)
    );

    Ok(())
}


// Optional: Add some basic integration tests for the CLI itself
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use crate::universal_rule::{UniversalRule, UniversalRuleFrontmatter}; // For creating test rule files

    /// Helper function to set up a temporary rules directory with specified rule files for testing.
    ///
    /// # Arguments
    /// * `rules_dir` - The `PathBuf` for the directory where rule files will be created.
    /// * `rules_data` - A slice of tuples, where each tuple contains:
    ///     - `name`: The base name of the rule file (without `.md`).
    ///     - `content`: The Markdown content of the rule.
    ///     - `globs_opt`: An optional vector of glob patterns for the frontmatter.
    ///                    Frontmatter is added if this is `Some` or if the name contains "cursor_specific".
    fn setup_rules_dir(rules_dir: &PathBuf, rules_data: &[(&str, &str, Option<Vec<&str>>)]) -> Result<()> {
        fs::create_dir_all(rules_dir)?;
        for (name, content, globs_opt) in rules_data {
            let mut file_content = String::new();
            // Add YAML frontmatter if globs are specified or for specific cursor rule types
            if globs_opt.is_some() || name.contains("cursor_specific") { // Add frontmatter for specific cases
                file_content.push_str("---\n");
                if let Some(globs) = globs_opt {
                    file_content.push_str("globs: [");
                    file_content.push_str(&globs.iter().map(|g| format!("\"{}\"", g)).collect::<Vec<String>>().join(", "));
                    file_content.push_str("]\n");
                }
                if name.contains("cursor_always") {
                    file_content.push_str("cursorRuleType: Always\n");
                }
                file_content.push_str("---\n");
            }
            file_content.push_str(content);
            fs::write(rules_dir.join(format!("{}.md", name)), file_content)?;
        }
        Ok(())
    }

    #[test]
    fn test_cli_cursor_output() -> Result<()> {
        let dir = tempdir()?;
        let rules_path = dir.path().join("test_rules");
        let output_path = dir.path().join("test_output");
        fs::create_dir_all(&output_path)?;

        setup_rules_dir(&rules_path, &[("cursor_rule1", "Cursor content 1", Some(vec!["*.rs"])), ("cursor_always", "Always content", None)])?;

        let cli = Cli {
            rules_dir: rules_path,
            agent: AgentName::Cursor,
            output_dir: output_path.clone(),
        };
        
        // Simulate running main's logic for Cursor
        let rules = discover_and_parse_rules(&cli.rules_dir)?;
        let converter = CursorConverter;
        converter.generate_rules(&rules, &cli.output_dir)?;

        let cursor_output_dir = output_path.join(".cursor").join("rules");
        assert!(cursor_output_dir.join("cursor_rule1.mdc").exists());
        let content = fs::read_to_string(cursor_output_dir.join("cursor_rule1.mdc"))?;
        assert!(content.contains("globs:"));
        assert!(content.contains("- \"*.rs\""));
        assert!(content.contains("Cursor content 1"));

        assert!(cursor_output_dir.join("cursor_always.mdc").exists());
        let always_content = fs::read_to_string(cursor_output_dir.join("cursor_always.mdc"))?;
        assert!(always_content.contains("alwaysApply: true"));
        assert!(always_content.contains("Always content"));
        
        Ok(())
    }

    #[test]
    fn test_cli_windsurf_output() -> Result<()> {
        let dir = tempdir()?;
        let rules_path = dir.path().join("test_rules_ws");
        let output_path = dir.path().join("test_output_ws");
        fs::create_dir_all(&output_path)?;

        // Global rule needs 'apply_globally: true' in its frontmatter to be treated as global
        // For simplicity, discover_and_parse_rules will need to handle this.
        // The test setup_rules_dir needs to be smarter or we simplify here.
        // Let's assume all rules are workspace for this CLI test to avoid complex setup.
        let mut rule_fm_global = UniversalRuleFrontmatter::default();
        rule_fm_global.apply_globally = true;
        rule_fm_global.description = Some("Global rule".to_string());

        let global_rule_file_content = "---\ndescription: Global rule\napplyGlobally: true\n---\nGlobal content";
        fs::create_dir_all(&rules_path)?;
        fs::write(rules_path.join("global_rule.md"), global_rule_file_content)?;
        setup_rules_dir(&rules_path, &[("ws_rule1", "WS content 1", Some(vec!["*.txt"]))])?;


        let cli = Cli {
            rules_dir: rules_path,
            agent: AgentName::Windsurf,
            output_dir: output_path.clone(),
        };

        let rules = discover_and_parse_rules(&cli.rules_dir)?;
        let converter = WindsurfConverter;
        converter.generate_rules(&rules, &cli.output_dir)?;

        assert!(output_path.join("global_rules.md").exists());
        let global_content = fs::read_to_string(output_path.join("global_rules.md"))?;
        assert!(global_content.contains("# Description: Global rule"));
        assert!(global_content.contains("Global content"));
        
        assert!(output_path.join(".windsurf").join("rules").join("ws_rule1.md").exists());
        let ws_content = fs::read_to_string(output_path.join(".windsurf").join("rules").join("ws_rule1.md"))?;
        assert!(ws_content.contains("# Globs: [\"*.txt\"]"));
        assert!(ws_content.contains("WS content 1"));
        Ok(())
    }

    #[test]
    fn test_cli_claude_output() -> Result<()> {
        let dir = tempdir()?;
        let rules_path = dir.path().join("test_rules_claude");
        let output_path = dir.path().join("test_output_claude");
        fs::create_dir_all(&output_path)?;

        setup_rules_dir(&rules_path, &[("claude_rule1", "Claude content 1", None), ("claude_rule2", "Claude content 2", None)])?;

        let cli = Cli {
            rules_dir: rules_path,
            agent: AgentName::Claude,
            output_dir: output_path.clone(),
        };
        
        let rules = discover_and_parse_rules(&cli.rules_dir)?;
        let converter = ClaudeConverter;
        converter.generate_rules(&rules, &cli.output_dir)?;

        let claude_file = output_path.join("CLAUDE.md");
        assert!(claude_file.exists());
        let content = fs::read_to_string(claude_file)?;
        assert!(content.contains("## Rule: claude_rule1"));
        assert!(content.contains("Claude content 1"));
        assert!(content.contains("## Rule: claude_rule2"));
        assert!(content.contains("Claude content 2"));
        assert!(content.contains("\n\n---\n\n"));
        Ok(())
    }
    
    #[test]
    fn test_rules_dir_not_exists() {
        let dir = tempdir().unwrap(); // Create a temp dir that exists
        let non_existent_rules_path = dir.path().join("non_existent_rules");
        // Do not create non_existent_rules_path

        // We need to run the binary or simulate its main execution flow
        // For now, let's just check the condition as it is in main()
        // A more robust test would use assert_cmd or similar.
        assert!(!non_existent_rules_path.exists());
        // The main function would call std::process::exit(1);
        // Testing that directly is tricky here. We're verifying the condition leading to it.
    }
}
