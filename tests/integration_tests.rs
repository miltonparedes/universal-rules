use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command; // Used to get the binary path

use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use tempfile::{tempdir, TempDir}; // For creating temporary directories

// Helper function to get the path to the compiled binary
fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    // Adjust for debug or release builds if necessary, assuming debug for tests
    path.push(if cfg!(debug_assertions) { "debug" } else { "release" });
    path.push("rule_unifier_cli"); // The binary name from Cargo.toml
    path
}

// Struct to hold paths for a test setup
struct TestSetup {
    _temp_dir: TempDir, // Keep TempDir to ensure it's not dropped early
    rules_dir: PathBuf,
    output_dir: PathBuf,
}

// Test setup helper function
fn setup_test_environment(test_name: &str) -> TestSetup {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let rules_dir = temp_dir.path().join(format!("{}_rules", test_name));
    let output_dir = temp_dir.path().join(format!("{}_output", test_name));

    fs::create_dir_all(&rules_dir).expect("Failed to create test rules directory");
    fs::create_dir_all(&output_dir).expect("Failed to create test output directory");

    // Create sample rule files
    let common_rule_content = "---
description: A common rule.
apply_globally: true
---
This is a common rule for all agents.";
    let mut common_file = File::create(rules_dir.join("common.md")).unwrap();
    writeln!(common_file, "{}", common_rule_content).unwrap();

    let cursor_specific_content = "---
description: Cursor specific settings.
cursor_rule_type: Always
---
Apply this always for Cursor.";
    let mut cursor_file = File::create(rules_dir.join("cursor_specific.md")).unwrap();
    writeln!(cursor_file, "{}", cursor_specific_content).unwrap();

    let windsurf_specific_content = "---
description: Windsurf workspace rule.
globs: [\"*.rs\"]
---
For Rust files in Windsurf.";
    let mut windsurf_file = File::create(rules_dir.join("windsurf_specific.md")).unwrap();
    writeln!(windsurf_file, "{}", windsurf_specific_content).unwrap();
    
    let simple_claude_content = "---
description: A simple rule for Claude.
---
This is a simple rule.";
    let mut claude_file = File::create(rules_dir.join("claude_simple.md")).unwrap();
    writeln!(claude_file, "{}", simple_claude_content).unwrap();


    TestSetup {
        _temp_dir: temp_dir,
        rules_dir,
        output_dir,
    }
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::new(get_binary_path());
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("urules"))
        .stdout(predicate::str::contains("Unifies coding agent rules"))
        .stdout(predicate::str::contains("--rules-dir"))
        .stdout(predicate::str::contains("--agent"))
        .stdout(predicate::str::contains("--output-dir"))
        .stdout(predicate::str::contains("--no-gitignore"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::new(get_binary_path());
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_invalid_rules_directory() {
    let mut cmd = Command::new(get_binary_path());
    cmd.arg("--rules-dir")
        .arg("non_existent_dir_for_testing_urules")
        .arg("--agent")
        .arg("cursor");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Rules directory"));
}

#[test]
fn test_no_rules_found() {
    let temp_dir = tempdir().unwrap();
    let empty_rules_dir = temp_dir.path().join("empty_rules");
    fs::create_dir(&empty_rules_dir).unwrap();

    let mut cmd = Command::new(get_binary_path());
    cmd.arg("--rules-dir")
        .arg(&empty_rules_dir)
        .arg("--agent")
        .arg("cursor");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No rules found"));
}

#[test]
fn test_cursor_generation_and_gitignore() {
    let setup = setup_test_environment("cursor_gen");
    let mut cmd = Command::new(get_binary_path());
    cmd.arg("--agent")
        .arg("cursor")
        .arg("--rules-dir")
        .arg(&setup.rules_dir)
        .arg("--output-dir")
        .arg(&setup.output_dir);

    cmd.assert().success().stdout(
        predicate::str::contains("Rules generated successfully for Cursor")
            .and(predicate::str::contains(".cursor/rules")),
    );

    // Verify file creation
    let cursor_output_rules_dir = setup.output_dir.join(".cursor").join("rules");
    assert!(cursor_output_rules_dir.join("common.mdc").exists());
    assert!(cursor_output_rules_dir.join("cursor_specific.mdc").exists());
    assert!(!cursor_output_rules_dir.join("windsurf_specific.mdc").exists()); // Ensure only relevant rules

    // Verify content of a key file
    let common_content = fs::read_to_string(cursor_output_rules_dir.join("common.mdc")).unwrap();
    assert!(common_content.contains("description: A common rule."));
    // apply_globally from universal rule does not directly map to a specific cursor field unless cursor_rule_type is "Always"
    // If cursor_rule_type is not "Always", apply_globally does not set alwaysApply:true for cursor.
    assert!(!common_content.contains("alwaysApply: true")); // Because cursor_rule_type wasn't "Always"
    assert!(common_content.contains("This is a common rule for all agents."));
    
    let cursor_specific_content_check = fs::read_to_string(cursor_output_rules_dir.join("cursor_specific.mdc")).unwrap();
    assert!(cursor_specific_content_check.contains("alwaysApply: true"));


    // Verify .gitignore
    let gitignore_path = setup.output_dir.join(".gitignore");
    assert!(gitignore_path.exists());
    let gitignore_content = fs::read_to_string(gitignore_path).unwrap();
    assert!(gitignore_content.contains(".cursor/"));
    assert!(gitignore_content.contains("# Added by urules"));
}

#[test]
fn test_windsurf_generation_and_gitignore() {
    let setup = setup_test_environment("windsurf_gen");
    let mut cmd = Command::new(get_binary_path());
    cmd.arg("--agent")
        .arg("windsurf")
        .arg("--rules-dir")
        .arg(&setup.rules_dir)
        .arg("--output-dir")
        .arg(&setup.output_dir);

    cmd.assert().success().stdout(
        predicate::str::contains("Rules generated successfully for Windsurf"),
    );

    // Verify file creation
    assert!(setup.output_dir.join("global_rules.md").exists());
    let windsurf_workspace_dir = setup.output_dir.join(".windsurf").join("rules");
    assert!(windsurf_workspace_dir.join("windsurf_specific.md").exists());
    assert!(!windsurf_workspace_dir.join("cursor_specific.md").exists());

    // Verify content
    let global_content = fs::read_to_string(setup.output_dir.join("global_rules.md")).unwrap();
    assert!(global_content.contains("# Description: A common rule."));
    assert!(global_content.contains("This is a common rule for all agents."));
    // Global rules from other files if apply_globally: true
    // Our cursor_specific.md had cursor_rule_type: Always, but not apply_globally for windsurf.
    // So it should NOT be in global_rules.md for windsurf.
    assert!(!global_content.contains("Apply this always for Cursor."));


    let ws_specific_content = fs::read_to_string(windsurf_workspace_dir.join("windsurf_specific.md")).unwrap();
    assert!(ws_specific_content.contains("# Description: Windsurf workspace rule."));
    assert!(ws_specific_content.contains("# Globs: [\"*.rs\"]"));
    assert!(ws_specific_content.contains("For Rust files in Windsurf."));

    // Verify .gitignore
    let gitignore_path = setup.output_dir.join(".gitignore");
    assert!(gitignore_path.exists());
    let gitignore_content = fs::read_to_string(gitignore_path).unwrap();
    assert!(gitignore_content.contains("global_rules.md"));
    assert!(gitignore_content.contains(".windsurf/"));
}

#[test]
fn test_claude_generation_and_gitignore() {
    let setup = setup_test_environment("claude_gen");
    let mut cmd = Command::new(get_binary_path());
    cmd.arg("--agent")
        .arg("claude")
        .arg("--rules-dir")
        .arg(&setup.rules_dir)
        .arg("--output-dir")
        .arg(&setup.output_dir);

    cmd.assert().success().stdout(
        predicate::str::contains("Rules generated successfully for Claude")
            .and(predicate::str::contains("CLAUDE.md")),
    );

    // Verify file creation and content
    let claude_file_path = setup.output_dir.join("CLAUDE.md");
    assert!(claude_file_path.exists());
    let claude_content = fs::read_to_string(claude_file_path).unwrap();
    assert!(claude_content.contains("## Rule: common"));
    assert!(claude_content.contains("A common rule."));
    assert!(claude_content.contains("This is a common rule for all agents."));
    assert!(claude_content.contains("---")); // Separator
    assert!(claude_content.contains("## Rule: cursor_specific"));
    assert!(claude_content.contains("Cursor specific settings."));
    assert!(claude_content.contains("Apply this always for Cursor."));
    assert!(claude_content.contains("## Rule: windsurf_specific"));
    assert!(claude_content.contains("Windsurf workspace rule."));
    assert!(claude_content.contains("For Rust files in Windsurf."));
    assert!(claude_content.contains("## Rule: claude_simple"));
    assert!(claude_content.contains("A simple rule for Claude."));
    assert!(claude_content.contains("This is a simple rule."));


    // Verify .gitignore
    let gitignore_path = setup.output_dir.join(".gitignore");
    assert!(gitignore_path.exists());
    let gitignore_content = fs::read_to_string(gitignore_path).unwrap();
    assert!(gitignore_content.contains("CLAUDE.md"));
}

#[test]
fn test_no_gitignore_flag() {
    let setup = setup_test_environment("no_git");
    let mut cmd = Command::new(get_binary_path());
    cmd.arg("--agent")
        .arg("cursor") // Any agent
        .arg("--rules-dir")
        .arg(&setup.rules_dir)
        .arg("--output-dir")
        .arg(&setup.output_dir)
        .arg("--no-gitignore");

    cmd.assert().success();

    // Verify .gitignore is NOT created
    let gitignore_path = setup.output_dir.join(".gitignore");
    assert!(!gitignore_path.exists());
}
