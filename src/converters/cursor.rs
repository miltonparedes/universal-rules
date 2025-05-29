// src/converters/cursor.rs

use super::RuleConverter;
use crate::universal_rule::UniversalRule;
use anyhow::{Context, Result};
use serde::Serialize;
use serde_yaml;
use std::fmt::Debug;
use std::fs;
use std::path::Path; // Required for derive(Debug) on MdcFrontmatter

/// Represents the YAML frontmatter structure for Cursor.ai's `.mdc` rule files.
///
/// Fields are serialized to `camelCase` as expected by Cursor.
/// Optional fields are skipped during serialization if they are `None`.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MdcFrontmatter {
    /// An optional description of the rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A list of glob patterns for which this rule should be active or suggested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub globs: Option<Vec<String>>,

    /// If `true`, Cursor attempts to apply this rule automatically/globally.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_apply: Option<bool>,

    /// If `true`, this rule is intended to be explicitly requested or triggered by the agent/user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_requested: Option<bool>,
}

/// Converts a `UniversalRule`'s frontmatter and content into the format
/// expected by Cursor.ai, specifically `MdcFrontmatter` and the rule's body.
///
/// This is a helper function used by the `CursorConverter`.
fn convert_to_cursor_rule(universal_rule: &UniversalRule) -> (MdcFrontmatter, String) {
    let mut mdc_frontmatter = MdcFrontmatter {
        description: universal_rule.frontmatter.description.clone(),
        globs: universal_rule.frontmatter.globs.clone(),
        ..Default::default() // Initializes always_apply and agent_requested to None
    };

    // Map UniversalRule's cursor_rule_type to MdcFrontmatter fields
    match universal_rule.frontmatter.cursor_rule_type.as_deref() {
        Some("Always") => {
            mdc_frontmatter.always_apply = Some(true);
        }
        Some("AutoAttached") => {
            // For "AutoAttached", the presence of `globs` is usually sufficient.
            // No specific MdcFrontmatter boolean flag needs to be set unless Cursor's
            // interpretation requires it (e.g., agentRequested: false explicitly).
            // Currently, we leave agent_requested as None or its default.
        }
        Some("AgentRequested") => {
            mdc_frontmatter.agent_requested = Some(true);
        }
        Some("Manual") | None => {
            // "Manual" rules or those with no specified type typically don't set
            // `always_apply` or `agent_requested` to true. They might be picked up
            // by Cursor based on their presence and `globs`.
        }
        Some(other_type) => {
            // Log unknown types but treat them as "Manual" to avoid errors.
            eprintln!(
                "Warning: Unknown cursor_rule_type '{}' for rule '{}', treating as Manual.",
                other_type, universal_rule.name
            );
        }
    }

    (mdc_frontmatter, universal_rule.content.clone())
}

/// A `RuleConverter` implementation for generating Cursor.ai specific rule files (`.mdc`).
pub struct CursorConverter;

impl RuleConverter for CursorConverter {
    /// Generates Cursor-specific `.mdc` rule files from a list of `UniversalRule`s.
    ///
    /// Each `UniversalRule` is converted into an individual `.mdc` file named after the rule.
    /// These files are placed in a `.cursor/rules/` subdirectory within the specified `output_dir`.
    /// The content of each `.mdc` file includes YAML frontmatter derived from `MdcFrontmatter`
    /// and the rule's Markdown body.
    fn generate_rules(&self, rules: &[UniversalRule], output_dir: &Path) -> Result<()> {
        let cursor_rules_dir = output_dir.join(".cursor").join("rules");
        fs::create_dir_all(&cursor_rules_dir).with_context(|| {
            format!(
                "Failed to create .cursor/rules directory at {:?}",
                cursor_rules_dir
            )
        })?;

        for rule in rules {
            let (mdc_frontmatter, rule_content) = convert_to_cursor_rule(rule);

            // Serialize MdcFrontmatter to YAML, only if there are any fields to serialize.
            let frontmatter_yaml = if mdc_frontmatter.description.is_none()
                && mdc_frontmatter.globs.is_none()
                && mdc_frontmatter.always_apply.is_none()
                && mdc_frontmatter.agent_requested.is_none()
            {
                String::new() // Empty string if all fields are None
            } else {
                serde_yaml::to_string(&mdc_frontmatter).with_context(|| {
                    format!("Failed to serialize MdcFrontmatter for rule: {}", rule.name)
                })?
            };

            // Construct the final content for the .mdc file.
            // Only include --- separators if there is frontmatter.
            let mdc_content = if frontmatter_yaml.is_empty() {
                rule_content
            } else {
                format!("---\n{}---\n{}", frontmatter_yaml.trim_end(), rule_content)
            };

            let output_file_path = cursor_rules_dir.join(format!("{}.mdc", rule.name));
            fs::write(&output_file_path, mdc_content)
                .with_context(|| format!("Failed to write .mdc file for rule: {}", rule.name))?;
        }
        Ok(())
    }

    /// Provides a description of where the Cursor rules are generated.
    fn get_output_description(&self, output_dir: &Path) -> String {
        format!(
            "Cursor rules in {:?}",
            output_dir.join(".cursor").join("rules")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_rule::{UniversalRule, UniversalRuleFrontmatter};
    use tempfile::tempdir;

    /// Helper function to create `UniversalRule` instances for testing the Cursor converter.
    fn create_test_universal_rule(
        name: &str,
        description: Option<&str>,
        globs: Option<Vec<&str>>,
        cursor_rule_type: Option<&str>,
        content: &str,
    ) -> UniversalRule {
        UniversalRule {
            name: name.to_string(),
            frontmatter: UniversalRuleFrontmatter {
                description: description.map(String::from),
                globs: globs.map(|g| g.iter().map(|s| s.to_string()).collect()),
                apply_globally: false,
                cursor_rule_type: cursor_rule_type.map(String::from),
            },
            content: content.to_string(),
        }
    }

    /// Test the `RuleConverter` trait implementation for `CursorConverter`.
    #[test]
    fn test_cursor_converter_trait_impl() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = CursorConverter;

        let rules = vec![create_test_universal_rule(
            "trait_rule1",
            Some("Trait First rule"),
            Some(vec!["*.rs"]),
            Some("Always"),
            "Trait Rule 1 content",
        )];

        converter.generate_rules(&rules, output_path).unwrap();
        let rule1_path = output_path
            .join(".cursor")
            .join("rules")
            .join("trait_rule1.mdc");
        assert!(rule1_path.exists(), "Cursor rule file should be created.");
        let content1 = fs::read_to_string(rule1_path).unwrap();
        assert!(content1.contains("description: Trait First rule"));
        assert!(content1.contains("alwaysApply: true"));
        assert!(content1.ends_with("Trait Rule 1 content"));
    }

    /// Test mapping from `UniversalRule` with `cursor_rule_type: "Always"`.
    #[test]
    fn test_convert_to_cursor_rule_always() {
        let rule = create_test_universal_rule(
            "always_rule",
            Some("Always active"),
            None,
            Some("Always"),
            "Content for always rule",
        );
        let (frontmatter, content) = convert_to_cursor_rule(&rule);
        assert_eq!(frontmatter.description, Some("Always active".to_string()));
        assert_eq!(frontmatter.always_apply, Some(true));
        assert!(frontmatter.agent_requested.is_none());
        assert!(frontmatter.globs.is_none());
        assert_eq!(content, "Content for always rule");
    }

    /// Test mapping from `UniversalRule` with `cursor_rule_type: "AgentRequested"`.
    #[test]
    fn test_convert_to_cursor_rule_agent_requested() {
        let rule = create_test_universal_rule(
            "agent_rule",
            Some("Agent needs this"),
            Some(vec!["*.py"]),
            Some("AgentRequested"),
            "Content for agent rule",
        );
        let (frontmatter, content) = convert_to_cursor_rule(&rule);
        assert_eq!(
            frontmatter.description,
            Some("Agent needs this".to_string())
        );
        assert_eq!(frontmatter.globs, Some(vec!["*.py".to_string()]));
        assert_eq!(frontmatter.agent_requested, Some(true));
        assert!(frontmatter.always_apply.is_none());
        assert_eq!(content, "Content for agent rule");
    }

    /// Test mapping from `UniversalRule` with `cursor_rule_type: "AutoAttached"`.
    #[test]
    fn test_convert_to_cursor_rule_auto_attached() {
        let rule = create_test_universal_rule(
            "auto_attach_rule",
            Some("Auto attaches"),
            Some(vec!["*.ts"]),
            Some("AutoAttached"),
            "Content for auto-attach rule",
        );
        let (frontmatter, content) = convert_to_cursor_rule(&rule);
        assert_eq!(frontmatter.description, Some("Auto attaches".to_string()));
        assert_eq!(frontmatter.globs, Some(vec!["*.ts".to_string()]));
        assert!(
            frontmatter.agent_requested.is_none(),
            "AutoAttached should not imply agentRequested: true by default"
        );
        assert!(frontmatter.always_apply.is_none());
        assert_eq!(content, "Content for auto-attach rule");
    }

    /// Test mapping for "Manual" `cursor_rule_type` or when it's `None`.
    #[test]
    fn test_convert_to_cursor_rule_manual_or_none() {
        let rule_manual =
            create_test_universal_rule("manual_rule", None, None, Some("Manual"), "Manual content");
        let (fm_manual, _) = convert_to_cursor_rule(&rule_manual);
        assert!(fm_manual.always_apply.is_none());
        assert!(fm_manual.agent_requested.is_none());

        let rule_none =
            create_test_universal_rule("none_type_rule", None, None, None, "None type content");
        let (fm_none, _) = convert_to_cursor_rule(&rule_none);
        assert!(fm_none.always_apply.is_none());
        assert!(fm_none.agent_requested.is_none());
    }

    /// Test YAML serialization of `MdcFrontmatter` with various fields set.
    #[test]
    fn test_mdc_frontmatter_serialization() {
        let fm = MdcFrontmatter {
            description: Some("Test desc".to_string()),
            globs: Some(vec!["*.rs".to_string(), "*.toml".to_string()]),
            always_apply: Some(true),
            agent_requested: None,
        };
        let yaml = serde_yaml::to_string(&fm).unwrap();
        assert!(yaml.contains("description: Test desc"));
        assert!(yaml.contains("globs:"));
        assert!(yaml.contains("- \"*.rs\""));
        assert!(yaml.contains("- \"*.toml\""));
        assert!(yaml.contains("alwaysApply: true"));
        assert!(
            !yaml.contains("agentRequested:"),
            "agentRequested should be omitted as it's None"
        );
    }

    /// Test YAML serialization when only `agent_requested` is true.
    #[test]
    fn test_mdc_frontmatter_serialization_agent_requested() {
        let fm = MdcFrontmatter {
            description: Some("Req agent".to_string()),
            globs: None,
            always_apply: None,
            agent_requested: Some(true),
        };
        let yaml = serde_yaml::to_string(&fm).unwrap();
        assert!(yaml.contains("description: Req agent"));
        assert!(yaml.contains("agentRequested: true"));
        assert!(
            !yaml.contains("alwaysApply:"),
            "alwaysApply should be omitted"
        );
        assert!(!yaml.contains("globs:"), "globs should be omitted");
    }

    /// Test YAML serialization when all optional fields in `MdcFrontmatter` are `None`.
    /// Expects an empty YAML map `{}`.
    #[test]
    fn test_mdc_frontmatter_serialization_minimal() {
        let fm = MdcFrontmatter {
            description: None,
            globs: None,
            always_apply: None,
            agent_requested: None,
        };
        let yaml = serde_yaml::to_string(&fm).unwrap();
        assert_eq!(
            yaml.trim(),
            "{}",
            "Serialization of all-None MdcFrontmatter should be an empty map."
        );
    }

    /// Test the creation of `.mdc` files by `generate_rules`.
    #[test]
    fn test_generate_cursor_rules_creates_files() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        let converter = CursorConverter;

        let rules = vec![
            create_test_universal_rule(
                "rule1",
                Some("First rule"),
                Some(vec!["*.txt"]),
                Some("Always"),
                "Rule 1 content",
            ),
            create_test_universal_rule(
                "rule2",
                Some("Second rule"),
                None,
                Some("AgentRequested"),
                "Rule 2 content",
            ),
            create_test_universal_rule(
                // Rule with no frontmatter fields effectively
                "rule3",
                None,
                None,
                None,
                "Rule 3 content",
            ),
        ];

        converter.generate_rules(&rules, output_path).unwrap();

        let cursor_rules_dir = output_path.join(".cursor").join("rules");
        assert!(
            cursor_rules_dir.exists(),
            "Cursor rules directory should be created."
        );
        assert!(cursor_rules_dir.is_dir());

        let rule1_path = cursor_rules_dir.join("rule1.mdc");
        assert!(rule1_path.exists());
        let content1 = fs::read_to_string(rule1_path).unwrap();
        assert!(content1.contains("description: First rule"));
        assert!(content1.contains("globs:"));
        assert!(content1.contains("- \"*.txt\""));
        assert!(content1.contains("alwaysApply: true"));
        assert!(
            content1.contains("---"),
            "Frontmatter separator missing for rule1"
        );
        assert!(content1.ends_with("Rule 1 content"));

        let rule2_path = cursor_rules_dir.join("rule2.mdc");
        assert!(rule2_path.exists());
        let content2 = fs::read_to_string(rule2_path).unwrap();
        assert!(content2.contains("description: Second rule"));
        assert!(content2.contains("agentRequested: true"));
        assert!(!content2.contains("globs:"));
        assert!(content2.ends_with("Rule 2 content"));

        let rule3_path = cursor_rules_dir.join("rule3.mdc");
        assert!(rule3_path.exists());
        let content3 = fs::read_to_string(rule3_path).unwrap();
        // Rule 3 had no frontmatter, so .mdc should only contain content
        assert_eq!(
            content3, "Rule 3 content",
            "Rule3 content should be exactly as provided, no frontmatter block."
        );
        assert!(
            !content3.contains("---"),
            "No frontmatter separator should exist for rule3"
        );
    }
}
