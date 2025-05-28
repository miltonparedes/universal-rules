// src/universal_rule.rs

use serde::Deserialize;
use std::fmt::Debug;

/// Represents the YAML frontmatter of a universal rule file.
///
/// This structure holds metadata that defines how a rule should be processed
/// and applied by different AI coding agents.
#[derive(Deserialize, Debug)]
pub struct UniversalRuleFrontmatter {
    /// An optional human-readable description of the rule's purpose or behavior.
    /// This can be used for documentation or comments in the generated agent-specific rules.
    pub description: Option<String>,

    /// A list of glob patterns (e.g., `["*.rs", "src/utils/*.ts"]`) that specify
    /// which files this rule should apply to.
    /// This is primarily used by agents like Cursor for auto-attaching rules or
    /// by Windsurf for workspace rule targeting.
    /// If `None` or empty, the rule's applicability might be determined by other factors
    /// or it might be considered for manual invocation.
    pub globs: Option<Vec<String>>,

    /// If `true`, this rule is intended to be applied globally across a project or workspace.
    /// This is particularly relevant for the Windsurf converter, which separates rules
    /// into a global file and workspace-specific files.
    /// Defaults to `false` if not specified in the YAML frontmatter.
    #[serde(default)] // Ensures bool::default() (false) is used if not present in YAML
    pub apply_globally: bool,

    /// Specifies the type of rule for the Cursor.ai agent, influencing how it's
    /// categorized and activated within Cursor.
    /// Examples: "Always", "AutoAttached", "AgentRequested", "Manual".
    /// This field is specific to the Cursor conversion process.
    pub cursor_rule_type: Option<String>,
}

impl Default for UniversalRuleFrontmatter {
    /// Provides default values for `UniversalRuleFrontmatter`.
    /// This is used by `serde` when `#[serde(default)]` is specified for the struct
    /// or for individual fields, and when a frontmatter block is missing or empty.
    fn default() -> Self {
        UniversalRuleFrontmatter {
            description: None,
            globs: None,
            apply_globally: false, // Default behavior is not global application
            cursor_rule_type: None,
        }
    }
}

/// Represents a complete universal rule, combining its parsed frontmatter
/// and the main Markdown content of the rule.
///
/// This struct is the central representation of a rule after it has been
/// read and parsed from a `.md` file.
#[derive(Debug)]
pub struct UniversalRule {
    /// The name of the rule, typically derived from the stem of its filename
    /// (e.g., "my_custom_rule" from "my_custom_rule.md").
    /// This field is assigned during the rule parsing process, not from the frontmatter.
    pub name: String,

    /// The metadata associated with the rule, parsed from the YAML frontmatter block
    /// of the rule file.
    pub frontmatter: UniversalRuleFrontmatter,

    /// The main content of the rule, written in Markdown.
    /// This is the body of the rule file that follows the optional frontmatter section.
    /// It contains the actual instructions or prompts for the AI agent.
    pub content: String,
}
