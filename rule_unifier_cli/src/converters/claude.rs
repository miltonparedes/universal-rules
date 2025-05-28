// src/converters/claude.rs

use super::RuleConverter;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use crate::universal_rule::UniversalRule;

/// A `RuleConverter` implementation for generating a single `CLAUDE.md` file.
///
/// This converter concatenates all provided universal rules into a single Markdown
/// file, suitable for use as a comprehensive prompt or knowledge base for Claude.
/// Each rule's name and description (if available) are included as headings.
pub struct ClaudeConverter;

impl RuleConverter for ClaudeConverter {
    /// Generates a `CLAUDE.md` file by concatenating all `UniversalRule`s.
    ///
    /// If no rules are provided, no file is created. Otherwise, each rule's name
    /// is added as a Level 2 Markdown heading (`## Rule: {name}`), followed by its
    /// description (if any) and then its content. Rules are separated by a
    /// Markdown horizontal rule (`\n\n---\n\n`).
    /// The output file is named `CLAUDE.md` and placed in the specified `output_dir`.
    fn generate_rules(&self, rules: &[UniversalRule], output_dir: &Path) -> Result<()> {
        if rules.is_empty() {
            // Do not create an empty CLAUDE.md if there are no rules to process.
            return Ok(());
        }

        let mut claude_content_parts = Vec::new();

        for rule in rules {
            let mut rule_block = String::new();
            // Add rule name as a heading
            rule_block.push_str(&format!("## Rule: {}\n", rule.name));
            // Add description if available, followed by a blank line
            if let Some(desc) = &rule.frontmatter.description {
                rule_block.push_str(&format!("{}\n\n", desc));
            } else {
                // Ensure a blank line after the name heading even if no description
                rule_block.push_str("\n");
            }
            // Add the main rule content
            rule_block.push_str(&rule.content);
            claude_content_parts.push(rule_block);
        }

        // Join all individual rule blocks with a Markdown separator.
        // This also handles the case of a single rule (no separator needed).
        let final_claude_content = claude_content_parts.join("\n\n---\n\n");

        fs::write(output_dir.join("CLAUDE.md"), final_claude_content)
            .with_context(|| format!("Failed to write CLAUDE.md to {:?}", output_dir))?;

        Ok(())
    }

    /// Provides a description of where the Claude rules file is generated.
    fn get_output_description(&self, output_dir: &Path) -> String {
        format!("Claude rules in {:?}", output_dir.join("CLAUDE.md"))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_rule::{UniversalRule, UniversalRuleFrontmatter};
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Read;

    /// Helper function to create `UniversalRule` instances for testing the Claude converter.
    fn create_test_claude_rule(
        name: &str,
        content: &str,
        description: Option<&str>,
    ) -> UniversalRule {
        UniversalRule {
            name: name.to_string(),
            content: content.to_string(),
            frontmatter: UniversalRuleFrontmatter {
                description: description.map(String::from),
                globs: None,
                apply_globally: false, 
                cursor_rule_type: None, 
            },
        }
    }

    /// Test the `RuleConverter` trait implementation for `ClaudeConverter`.
    #[test]
    fn test_claude_converter_trait_impl() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = ClaudeConverter;

        let rules = vec![
            create_test_claude_rule("trait_rule", "Trait Content", Some("Trait Desc")),
        ];

        converter.generate_rules(&rules, output_path).unwrap();
        let claude_path = output_path.join("CLAUDE.md");
        assert!(claude_path.exists(), "CLAUDE.md file should be created.");
        let mut content = String::new();
        File::open(claude_path).unwrap().read_to_string(&mut content).unwrap();
        assert!(content.contains("## Rule: trait_rule\nTrait Desc\n\nTrait Content"));
    }

    /// Test generation of `CLAUDE.md` with multiple rules, checking content and separators.
    #[test]
    fn test_generate_claude_rules_multiple_rules() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = ClaudeConverter;

        let rules = vec![
            create_test_claude_rule("rule1", "Content for rule 1.", Some("Description for rule 1.")),
            create_test_claude_rule("rule2", "Content for rule 2.", None), // Rule without description
            create_test_claude_rule("rule3", "Content for rule 3.", Some("Description for rule 3.")),
        ];

        converter.generate_rules(&rules, output_path).unwrap();

        let claude_file_path = output_path.join("CLAUDE.md");
        assert!(claude_file_path.exists());

        let mut claude_content = String::new();
        File::open(claude_file_path).unwrap().read_to_string(&mut claude_content).unwrap();

        // Verify content of each rule
        assert!(claude_content.contains("## Rule: rule1\nDescription for rule 1.\n\nContent for rule 1."));
        assert!(claude_content.contains("## Rule: rule2\n\nContent for rule 2.")); // Expect double newline after name for no-description rule
        assert!(claude_content.contains("## Rule: rule3\nDescription for rule 3.\n\nContent for rule 3."));
        
        // Verify separator presence
        assert!(claude_content.contains("\n\n---\n\n"), "Separator between rules is missing.");
        
        // Verify order and exact structure of a segment
        let expected_block_for_rule1 = "## Rule: rule1\nDescription for rule 1.\n\nContent for rule 1.";
        let expected_block_for_rule2 = "## Rule: rule2\n\nContent for rule 2.";
        assert!(claude_content.contains(&format!("{}\n\n---\n\n{}", expected_block_for_rule1, expected_block_for_rule2)));
    }

    /// Test generation with a single rule, ensuring no separators are added.
    #[test]
    fn test_generate_claude_rules_single_rule() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = ClaudeConverter;
        let rules = vec![
            create_test_claude_rule("single_rule", "Single rule content.", Some("Desc for single."))
        ];
        converter.generate_rules(&rules, output_path).unwrap();

        let claude_file_path = output_path.join("CLAUDE.md");
        assert!(claude_file_path.exists());
        let mut claude_content = String::new();
        File::open(claude_file_path).unwrap().read_to_string(&mut claude_content).unwrap();

        assert!(claude_content.contains("## Rule: single_rule\nDesc for single.\n\nSingle rule content."));
        assert!(!claude_content.contains("\n\n---\n\n"), "Separator should not be present for a single rule."); 
    }

    /// Test behavior when no rules are provided; expects no file to be created.
    #[test]
    fn test_generate_claude_rules_no_rules() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = ClaudeConverter;
        let rules: Vec<UniversalRule> = vec![]; // Empty rule set

        converter.generate_rules(&rules, output_path).unwrap();

        let claude_file_path = output_path.join("CLAUDE.md");
        assert!(!claude_file_path.exists(), "CLAUDE.md should not be created if no rules are provided."); 
    }
    
    /// Test the specific formatting of rule name and description (with and without description).
    #[test]
    fn test_formatting_of_rule_name_and_description() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = ClaudeConverter;

        // Rule with description
        let rule_with_desc = create_test_claude_rule("desc_rule", "Content here.", Some("This is a description."));
        converter.generate_rules(&[rule_with_desc], output_path).unwrap();
        let mut content_with_desc = String::new();
        File::open(output_path.join("CLAUDE.md")).unwrap().read_to_string(&mut content_with_desc).unwrap();
        let expected_with_desc = "## Rule: desc_rule\nThis is a description.\n\nContent here.";
        assert_eq!(content_with_desc.trim(), expected_with_desc);
        
        // Clean up the file for the next test case within the same function
        fs::remove_file(output_path.join("CLAUDE.md")).unwrap();

        // Rule without description
        let rule_no_desc = create_test_claude_rule("no_desc_rule", "More content.", None);
        converter.generate_rules(&[rule_no_desc], output_path).unwrap();
        let mut content_no_desc = String::new();
        File::open(output_path.join("CLAUDE.md")).unwrap().read_to_string(&mut content_no_desc).unwrap();
        let expected_no_desc = "## Rule: no_desc_rule\n\nMore content."; // Note the expected double newline
        assert_eq!(content_no_desc.trim(), expected_no_desc);
    }
}
