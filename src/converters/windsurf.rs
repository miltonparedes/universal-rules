// src/converters/windsurf.rs

use super::RuleConverter;
use crate::universal_rule::UniversalRule;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// A `RuleConverter` implementation for generating Windsurf-compatible rule files.
///
/// Windsurf rules are typically organized into:
/// 1.  A `global_rules.md` file for rules that apply across the entire workspace.
/// 2.  Individual rule files within a `.windsurf/rules/` directory for workspace-specific
///     or file-type-specific rules.
pub struct WindsurfConverter;

impl RuleConverter for WindsurfConverter {
    /// Generates Windsurf rule files from a list of `UniversalRule`s.
    ///
    /// Rules marked with `apply_globally: true` in their frontmatter are concatenated
    /// into `global_rules.md` in the `output_dir`. Other rules are saved as individual
    /// `.md` files (named after the rule) within an `output_dir/.windsurf/rules/` subdirectory.
    /// Descriptions and globs from the frontmatter are prepended as comments in the
    /// generated rule files.
    fn generate_rules(&self, rules: &[UniversalRule], output_dir: &Path) -> Result<()> {
        let mut global_rules_content = String::new();
        let mut has_workspace_rules = false; // Track if any non-global rules exist

        // First pass: collect global rules and identify if workspace rules are present
        for rule in rules {
            if rule.frontmatter.apply_globally {
                if let Some(desc) = &rule.frontmatter.description {
                    global_rules_content.push_str(&format!("# Description: {}\n", desc));
                }
                global_rules_content.push_str(&rule.content);
                global_rules_content.push_str("\n\n---\n\n"); // Markdown separator for rules
            } else {
                has_workspace_rules = true;
            }
        }

        // Write global_rules.md if any global rule content was generated
        if !global_rules_content.is_empty() {
            // Remove the trailing separator from the last rule
            if global_rules_content.ends_with("\n\n---\n\n") {
                global_rules_content.truncate(global_rules_content.len() - "\n\n---\n\n".len());
            }
            fs::write(output_dir.join("global_rules.md"), &global_rules_content)
                .with_context(|| format!("Failed to write global_rules.md to {:?}", output_dir))?;
        }

        // Process and write workspace-specific rules if any exist
        if has_workspace_rules {
            let windsurf_workspace_rules_dir = output_dir.join(".windsurf").join("rules");
            fs::create_dir_all(&windsurf_workspace_rules_dir).with_context(|| {
                format!(
                    "Failed to create Windsurf workspace rules directory at {:?}",
                    windsurf_workspace_rules_dir
                )
            })?;

            for rule in rules {
                if !rule.frontmatter.apply_globally {
                    let mut individual_rule_content = String::new();
                    // Prepend description as a comment if available
                    if let Some(desc) = &rule.frontmatter.description {
                        individual_rule_content.push_str(&format!("# Description: {}\n", desc));
                    }
                    // Prepend globs as a comment if available
                    if let Some(globs) = &rule.frontmatter.globs {
                        if !globs.is_empty() {
                            individual_rule_content.push_str(&format!("# Globs: {:?}\n", globs));
                        }
                    }
                    // Add a newline after comments if any were added, before rule content
                    if !individual_rule_content.is_empty() {
                        individual_rule_content.push('\n');
                    }
                    individual_rule_content.push_str(&rule.content);

                    let output_file_path =
                        windsurf_workspace_rules_dir.join(format!("{}.md", rule.name));
                    fs::write(&output_file_path, individual_rule_content).with_context(|| {
                        format!(
                            "Failed to write Windsurf workspace rule file for '{}' to {:?}",
                            rule.name, output_file_path
                        )
                    })?;
                }
            }
        }
        Ok(())
    }

    /// Provides a description of where the Windsurf rules are generated.
    fn get_output_description(&self, output_dir: &Path) -> String {
        format!(
            "Windsurf rules in {:?} (global) and potentially in {:?}",
            output_dir.join("global_rules.md"),
            output_dir.join(".windsurf").join("rules")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_rule::{UniversalRule, UniversalRuleFrontmatter};
    use std::fs::File;
    use std::io::Read;
    use tempfile::tempdir;

    /// Helper function to create `UniversalRule` instances for testing the Windsurf converter.
    fn create_test_rule(
        name: &str,
        content: &str,
        apply_globally: bool,
        description: Option<&str>,
        globs: Option<Vec<&str>>,
    ) -> UniversalRule {
        UniversalRule {
            name: name.to_string(),
            content: content.to_string(),
            frontmatter: UniversalRuleFrontmatter {
                description: description.map(String::from),
                globs: globs.map(|g| g.iter().map(|s| s.to_string()).collect()),
                apply_globally,
                cursor_rule_type: None,
            },
        }
    }

    /// Test the `RuleConverter` trait implementation for `WindsurfConverter`.
    /// Checks creation of both global and workspace rule files.
    #[test]
    fn test_windsurf_converter_trait_impl() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = WindsurfConverter;

        let rules = vec![
            create_test_rule(
                "trait_global",
                "Trait Global Content",
                true,
                Some("Trait Global Desc"),
                None,
            ),
            create_test_rule(
                "trait_ws",
                "Trait WS Content",
                false,
                Some("Trait WS Desc"),
                Some(vec!["*.test"]),
            ),
        ];

        converter.generate_rules(&rules, output_path).unwrap();

        let global_path = output_path.join("global_rules.md");
        assert!(global_path.exists(), "Global rules file should be created.");
        let mut global_content = String::new();
        File::open(global_path)
            .unwrap()
            .read_to_string(&mut global_content)
            .unwrap();
        assert!(global_content.contains("# Description: Trait Global Desc"));
        assert!(global_content.contains("Trait Global Content"));

        let ws_path = output_path
            .join(".windsurf")
            .join("rules")
            .join("trait_ws.md");
        assert!(ws_path.exists(), "Workspace rule file should be created.");
        let mut ws_content = String::new();
        File::open(ws_path)
            .unwrap()
            .read_to_string(&mut ws_content)
            .unwrap();
        assert!(ws_content.contains("# Description: Trait WS Desc"));
        assert!(ws_content.contains("# Globs: [\"*.test\"]"));
        assert!(ws_content.contains("Trait WS Content"));
    }

    /// Test generation with a mix of global and workspace rules.
    #[test]
    fn test_generate_windsurf_rules_mixed() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = WindsurfConverter;

        let rules = vec![
            create_test_rule(
                "global1",
                "Global rule 1 content",
                true,
                Some("Global desc 1"),
                None,
            ),
            create_test_rule(
                "workspace1",
                "Workspace rule 1 content",
                false,
                Some("WS desc 1"),
                Some(vec!["*.rs"]),
            ),
            create_test_rule("global2", "Global rule 2 content", true, None, None), // Global without description
            create_test_rule(
                "workspace2",
                "Workspace rule 2 content",
                false,
                None,
                Some(vec!["*.ts", "*.js"]),
            ), // Workspace without description
            create_test_rule("workspace3_no_meta", "WS rule 3 no meta", false, None, None), // Workspace with no metadata
        ];

        converter.generate_rules(&rules, output_path).unwrap();

        // Verify global_rules.md
        let global_rules_path = output_path.join("global_rules.md");
        assert!(global_rules_path.exists());
        let mut global_content = String::new();
        File::open(global_rules_path)
            .unwrap()
            .read_to_string(&mut global_content)
            .unwrap();

        assert!(global_content.contains("# Description: Global desc 1"));
        assert!(global_content.contains("Global rule 1 content"));
        assert!(global_content.contains("Global rule 2 content")); // Should not have "# Description:"
        assert!(
            global_content.contains("\n\n---\n\n"),
            "Separator missing between global rules."
        );
        assert!(
            !global_content.contains("Workspace rule 1 content"),
            "Workspace content found in global file."
        );

        // Verify workspace rules
        let ws_rules_dir = output_path.join(".windsurf").join("rules");
        assert!(
            ws_rules_dir.exists(),
            "Workspace rules directory not created."
        );

        // Workspace Rule 1
        let ws1_path = ws_rules_dir.join("workspace1.md");
        assert!(ws1_path.exists());
        let mut ws1_content = String::new();
        File::open(ws1_path)
            .unwrap()
            .read_to_string(&mut ws1_content)
            .unwrap();
        assert!(ws1_content.contains("# Description: WS desc 1\n"));
        assert!(ws1_content.contains("# Globs: [\"*.rs\"]\n\n")); // Expect newline after comments
        assert!(ws1_content.ends_with("Workspace rule 1 content"));

        // Workspace Rule 2 (no description)
        let ws2_path = ws_rules_dir.join("workspace2.md");
        assert!(ws2_path.exists());
        let mut ws2_content = String::new();
        File::open(ws2_path)
            .unwrap()
            .read_to_string(&mut ws2_content)
            .unwrap();
        assert!(!ws2_content.contains("# Description:"));
        assert!(ws2_content.contains("# Globs: [\"*.ts\", \"*.js\"]\n\n"));
        assert!(ws2_content.ends_with("Workspace rule 2 content"));

        // Workspace Rule 3 (no metadata)
        let ws3_path = ws_rules_dir.join("workspace3_no_meta.md");
        assert!(ws3_path.exists());
        let mut ws3_content = String::new();
        File::open(ws3_path)
            .unwrap()
            .read_to_string(&mut ws3_content)
            .unwrap();
        assert!(!ws3_content.contains("# Description:"));
        assert!(!ws3_content.contains("# Globs:"));
        assert_eq!(ws3_content, "WS rule 3 no meta"); // Content should be exactly this
    }

    /// Test generation when only global rules are provided.
    #[test]
    fn test_generate_windsurf_rules_only_global() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = WindsurfConverter;
        let rules = vec![
            create_test_rule("global_only1", "Content G1", true, Some("Desc G1"), None),
            create_test_rule("global_only2", "Content G2", true, None, None),
        ];
        converter.generate_rules(&rules, output_path).unwrap();

        let global_rules_path = output_path.join("global_rules.md");
        assert!(global_rules_path.exists());
        let mut global_content = String::new();
        File::open(global_rules_path)
            .unwrap()
            .read_to_string(&mut global_content)
            .unwrap();
        assert!(global_content.contains("# Description: Desc G1"));
        assert!(global_content.contains("Content G1"));
        assert!(global_content.contains("Content G2"));
        assert!(global_content.contains("\n\n---\n\n"));

        let ws_rules_dir = output_path.join(".windsurf").join("rules");
        assert!(
            !ws_rules_dir.exists(),
            "Workspace rules directory should not be created if only global rules exist."
        );
    }

    /// Test generation when only workspace rules are provided.
    #[test]
    fn test_generate_windsurf_rules_only_workspace() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = WindsurfConverter;
        let rules = vec![create_test_rule(
            "ws_only1",
            "Content WS1",
            false,
            Some("Desc WS1"),
            Some(vec!["*.py"]),
        )];
        converter.generate_rules(&rules, output_path).unwrap();

        let global_rules_path = output_path.join("global_rules.md");
        assert!(
            !global_rules_path.exists(),
            "Global rules file should not be created if only workspace rules exist."
        );

        let ws_rules_dir = output_path.join(".windsurf").join("rules");
        assert!(ws_rules_dir.exists());
        let ws1_path = ws_rules_dir.join("ws_only1.md");
        assert!(ws1_path.exists());
        let mut ws1_content = String::new();
        File::open(ws1_path)
            .unwrap()
            .read_to_string(&mut ws1_content)
            .unwrap();
        assert!(ws1_content.contains("# Description: Desc WS1"));
        assert!(ws1_content.contains("# Globs: [\"*.py\"]"));
        assert!(ws1_content.ends_with("Content WS1"));
    }

    /// Test behavior when no rules are provided.
    #[test]
    fn test_generate_windsurf_rules_no_rules() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = WindsurfConverter;
        let rules: Vec<UniversalRule> = vec![];
        converter.generate_rules(&rules, output_path).unwrap();

        assert!(
            !output_path.join("global_rules.md").exists(),
            "Global rules file should not exist for no rules."
        );
        assert!(
            !output_path.join(".windsurf").join("rules").exists(),
            "Workspace rules directory should not exist for no rules."
        );
    }

    /// Test that the Markdown separator in `global_rules.md` is correctly trimmed.
    #[test]
    fn test_global_rule_separator_trimmed_correctly() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = WindsurfConverter;

        // Single global rule
        let rules_single = vec![create_test_rule("g1", "content1", true, None, None)];
        converter
            .generate_rules(&rules_single, output_path)
            .unwrap();
        let global_rules_path = output_path.join("global_rules.md");
        let mut global_content_single = String::new();
        File::open(&global_rules_path)
            .unwrap()
            .read_to_string(&mut global_content_single)
            .unwrap();
        assert_eq!(
            global_content_single.trim_end(),
            "content1",
            "Single global rule should not have a trailing separator."
        );

        // Multiple global rules
        let rules_multiple = vec![
            create_test_rule("g1", "content1", true, None, None),
            create_test_rule("g2", "content2", true, None, None),
        ];
        converter
            .generate_rules(&rules_multiple, output_path)
            .unwrap(); // Overwrites previous file
        let mut global_content_multiple = String::new();
        File::open(&global_rules_path)
            .unwrap()
            .read_to_string(&mut global_content_multiple)
            .unwrap();
        assert!(
            global_content_multiple.contains("content1\n\n---\n\ncontent2"),
            "Separator missing between multiple global rules."
        );
        assert!(
            !global_content_multiple.ends_with("\n\n---\n\n"),
            "Multiple global rules should not have a trailing separator."
        );
    }
}
