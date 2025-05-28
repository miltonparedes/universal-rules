// src/rule_parser.rs

use std::fs;
use std::path::Path; // PathBuf is not used directly here, but often useful with Path
use walkdir::WalkDir;
use serde_yaml;
use anyhow::{Context, Result, anyhow};
use crate::universal_rule::{UniversalRule, UniversalRuleFrontmatter};

/// Parses a single universal rule file from the given `file_path`.
///
/// The function reads the file content, attempts to extract a YAML frontmatter block
/// (enclosed by `---` lines at the beginning of the file), and parses it into
/// `UniversalRuleFrontmatter`. The remaining content is considered the Markdown body
/// of the rule. The rule's name is derived from the file stem of `file_path`.
///
/// If no frontmatter is found, default `UniversalRuleFrontmatter` values are used.
///
/// # Arguments
/// * `file_path` - A reference to a `Path` pointing to the rule file.
///
/// # Returns
/// A `Result` containing the parsed `UniversalRule` on success, or an `anyhow::Error`
/// if reading the file, parsing frontmatter, or deriving the name fails.
pub fn parse_rule_file(file_path: &Path) -> Result<UniversalRule> {
    let file_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read rule file: {:?}", file_path))?;

    // Attempt to split the file content into frontmatter and main content.
    // Frontmatter is expected to be enclosed by '---' at the start and end.
    let (frontmatter_str, content_str) =
        if file_content.starts_with("---") {
            let mut parts = file_content.splitn(3, "---");
            parts.next(); // Skip the part before the first '---' (should be empty)
            let fm_block = parts.next().unwrap_or("").trim(); // The YAML block
            let main_content = parts.next().unwrap_or("").trim_start(); // The rest of the file
            (fm_block, main_content)
        } else {
            // No frontmatter detected, treat the entire file as content.
            ("", file_content.as_str())
        };

    // Parse the extracted frontmatter string into UniversalRuleFrontmatter.
    // If the frontmatter string is empty, use default values.
    let frontmatter: UniversalRuleFrontmatter = if frontmatter_str.is_empty() {
        UniversalRuleFrontmatter::default()
    } else {
        serde_yaml::from_str(frontmatter_str)
            .with_context(|| format!("Failed to parse YAML frontmatter for rule file: {:?}", file_path))?
    };

    // Derive the rule name from the file's stem (filename without extension).
    let name = file_path
        .file_stem()
        .ok_or_else(|| anyhow!("Failed to get file stem for {:?}", file_path))?
        .to_string_lossy() // Convert OsStr to String, lossily if necessary.
        .into_owned();

    Ok(UniversalRule {
        name,
        frontmatter,
        content: content_str.to_string(),
    })
}

/// Discovers and parses all universal rule files (Markdown `.md` files)
/// within a given directory and its subdirectories.
///
/// This function recursively walks through the `rules_dir`, identifies files
/// with the `.md` extension, and attempts to parse each one using `parse_rule_file`.
/// Errors encountered during the parsing of individual files are printed to `stderr`,
/// but the function continues to process other files.
///
/// # Arguments
/// * `rules_dir` - A reference to a `Path` for the directory to scan for rule files.
///
/// # Returns
/// A `Result` containing a `Vec<UniversalRule>` of all successfully parsed rules,
/// or an `anyhow::Error` if there's an issue walking the directory itself (though
/// individual file parsing errors are handled internally by logging).
pub fn discover_and_parse_rules(rules_dir: &Path) -> Result<Vec<UniversalRule>> {
    let mut rules = Vec::new();
    for entry in WalkDir::new(rules_dir)
        .into_iter()
        .filter_map(|e| e.ok()) // Filter out directory reading errors, processing valid entries.
    {
        let path = entry.path();
        // Check if the entry is a file and has a ".md" extension.
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            match parse_rule_file(path) {
                Ok(rule) => rules.push(rule),
                Err(e) => {
                    // Log errors for individual file parsing but continue with others.
                    eprintln!("Failed to parse rule file {:?}: {}", path, e);
                }
            }
        }
    }
    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    // PathBuf is used in tests for creating paths in temp directories
    use std::io::Write;
    use tempfile::tempdir;

    /// Test parsing a rule file that includes a valid YAML frontmatter block.
    #[test]
    fn test_parse_rule_file_with_frontmatter() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_rule.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            "---
description: A test rule
globs: [\"*.rs\"]
apply_globally: true
cursor_rule_type: \"lint\"
---
Rule content here."
        )
        .unwrap();

        let rule = parse_rule_file(&file_path).unwrap();
        assert_eq!(rule.name, "test_rule");
        assert_eq!(rule.frontmatter.description, Some("A test rule".to_string()));
        assert_eq!(rule.frontmatter.globs, Some(vec!["*.rs".to_string()]));
        assert_eq!(rule.frontmatter.apply_globally, true);
        assert_eq!(rule.frontmatter.cursor_rule_type, Some("lint".to_string()));
        assert_eq!(rule.content, "Rule content here.");
    }

    /// Test parsing a rule file that does not contain any frontmatter.
    /// Expects default frontmatter values and the entire file as content.
    #[test]
    fn test_parse_rule_file_without_frontmatter() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_rule_no_fm.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Just content here.").unwrap();

        let rule = parse_rule_file(&file_path).unwrap();
        assert_eq!(rule.name, "test_rule_no_fm");
        assert!(rule.frontmatter.description.is_none());
        assert!(rule.frontmatter.globs.is_none());
        assert_eq!(rule.frontmatter.apply_globally, false); // Default behavior
        assert!(rule.frontmatter.cursor_rule_type.is_none());
        assert_eq!(rule.content, "Just content here.");
    }

    /// Test parsing a rule file with an empty frontmatter block (just `---` lines).
    /// Expects default frontmatter values.
    #[test]
    fn test_parse_rule_file_empty_frontmatter() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_rule_empty_fm.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            "---
---
Content after empty frontmatter."
        )
        .unwrap();

        let rule = parse_rule_file(&file_path).unwrap();
        assert_eq!(rule.name, "test_rule_empty_fm");
        assert!(rule.frontmatter.description.is_none());
        assert!(rule.frontmatter.globs.is_none());
        assert_eq!(rule.frontmatter.apply_globally, false); // Default behavior
        assert!(rule.frontmatter.cursor_rule_type.is_none());
        assert_eq!(rule.content, "Content after empty frontmatter.");
    }
    
    /// Test parsing a rule file with malformed YAML in its frontmatter.
    /// Expects a parsing error.
    #[test]
    fn test_parse_rule_file_malformed_frontmatter() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_rule_malformed_fm.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            "---
description: A test rule
globs: [\"*.rs\" 
apply_globally: true 
cursor_rule_type: \"lint\"
---
Rule content here."
        )
        .unwrap(); // Malformed YAML: globs array is not closed

        let result = parse_rule_file(&file_path);
        assert!(result.is_err(), "Parsing should fail for malformed frontmatter");
    }

    /// Test discovering and parsing multiple rule files from a directory structure.
    /// Includes valid rules, a rule without frontmatter, a rule with malformed frontmatter (should be skipped),
    /// a non-Markdown file (should be ignored), and a rule in a subdirectory.
    #[test]
    fn test_discover_and_parse_rules() {
        let dir = tempdir().unwrap();
        let rules_subdir = dir.path().join(".rules");
        fs::create_dir(&rules_subdir).unwrap();

        // Rule 1: Valid with frontmatter
        let mut file1 = File::create(rules_subdir.join("rule1.md")).unwrap();
        writeln!(file1, "---\ndescription: Rule 1\n---\nContent 1").unwrap();

        // Rule 2: Valid, no frontmatter
        let mut file2 = File::create(rules_subdir.join("rule2.md")).unwrap();
        writeln!(file2, "Content 2").unwrap();
        
        // Rule 3: Invalid frontmatter (should be skipped, error printed)
        let mut file3 = File::create(rules_subdir.join("rule3.md")).unwrap();
        writeln!(file3, "---\ndescription: Rule 3\nglobs: [\"*.txt\"\n---\nContent 3").unwrap();

        // File 4: Not a Markdown file (should be ignored)
        let mut file4 = File::create(rules_subdir.join("notes.txt")).unwrap();
        writeln!(file4, "Some notes, not a rule.").unwrap();
        
        // Rule 5: Valid rule in a nested directory
        let sub_subdir = rules_subdir.join("nested");
        fs::create_dir(&sub_subdir).unwrap();
        let mut file5 = File::create(sub_subdir.join("rule5.md")).unwrap();
        writeln!(file5, "---\ndescription: Rule 5 in nested dir\n---\nContent 5").unwrap();


        let rules = discover_and_parse_rules(&rules_subdir).unwrap();
        // Expect rule1, rule2, and rule5 to be parsed. rule3 has malformed YAML.
        assert_eq!(rules.len(), 3, "Expected 3 valid rules to be parsed.");

        assert!(rules.iter().any(|r| r.name == "rule1" && r.frontmatter.description == Some("Rule 1".to_string())));
        assert!(rules.iter().any(|r| r.name == "rule2" && r.content == "Content 2" && r.frontmatter.description.is_none()));
        assert!(rules.iter().any(|r| r.name == "rule5" && r.frontmatter.description == Some("Rule 5 in nested dir".to_string())));
    }
}
