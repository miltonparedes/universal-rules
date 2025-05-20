// src/gitignore_manager.rs

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use anyhow::{Context, Result};

// Assuming AgentName will be accessible from main.rs or a shared module.
// This path will work if main.rs is part of the crate root (e.g. lib.rs then main.rs)
// or if main.rs is effectively the crate root for a binary crate.
// If AgentName is moved to its own module, this path will need to change.
use crate::AgentName; // Corrected path assuming AgentName is pub in main.rs or lib.rs

const GITIGNORE_HEADER: &str = "# Added by urules";
const GITIGNORE_FOOTER: &str = "# End urules section"; // Optional: for more robust section management

/// Updates the .gitignore file in the specified output directory to include
/// patterns related to the generated agent files, unless they are already present
/// within a urules-managed section or generally.
///
/// # Arguments
/// * `output_dir` - The directory where the .gitignore file is located (or should be created).
/// * `agent_name` - The `AgentName` enum indicating which agent's files were generated.
///
/// # Returns
/// A `Result` indicating success or failure of the .gitignore update operation.
pub fn update_gitignore(output_dir: &Path, agent_name: &AgentName) -> Result<()> {
    let patterns_to_add: Vec<String> = match agent_name {
        AgentName::Cursor => vec![".cursor/".to_string()],
        AgentName::Windsurf => vec!["global_rules.md".to_string(), ".windsurf/".to_string()],
        AgentName::Claude => vec!["CLAUDE.md".to_string()],
    };

    let gitignore_path = output_dir.join(".gitignore");
    let mut existing_lines = HashSet::new();
    let mut pre_content = String::new(); // Content before urules section
    let mut urules_section_content = String::new(); // Content within urules section
    let mut post_content = String::new(); // Content after urules section
    
    let mut in_urules_section = false;
    let mut urules_header_found = false;

    if gitignore_path.exists() {
        let file = File::open(&gitignore_path)
            .with_context(|| format!("Failed to open .gitignore file at {:?}", gitignore_path))?;
        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            let line = line_result.with_context(|| "Failed to read line from .gitignore")?;
            
            if line.trim() == GITIGNORE_HEADER {
                in_urules_section = true;
                urules_header_found = true;
                // Don't add header to any specific part yet, reconstruct later
                continue; 
            } else if line.trim() == GITIGNORE_FOOTER && in_urules_section {
                in_urules_section = false;
                // Don't add footer to any specific part yet
                continue;
            }

            if urules_header_found && in_urules_section { // Check urules_header_found to ensure we are after the header
                urules_section_content.push_str(&line);
                urules_section_content.push('\n');
                existing_lines.insert(line.trim().to_string());
            } else if !urules_header_found { // Content before our section (or if no section at all)
                pre_content.push_str(&line);
                pre_content.push('\n');
                existing_lines.insert(line.trim().to_string()); // Also consider lines outside our section
            } else { // Content after our section
                post_content.push_str(&line);
                post_content.push('\n');
                // Do not add these to existing_lines for purposes of *our* section management
            }
        }
    }
    
    // If urules_header_found is true, but we finished reading and in_urules_section is still true,
    // it means there was no footer. All remaining content is part of our section.
    // This case is implicitly handled as urules_section_content would contain everything after header.

    let mut final_new_patterns = Vec::new();
    for pattern_to_check in &patterns_to_add {
        let trimmed_pattern = pattern_to_check.trim_matches('/');
        // Check variations: exact, /dir, dir/, /dir/
        let variations = [
            pattern_to_check.clone(),
            format!("/{}", trimmed_pattern),
            format!("{}/", trimmed_pattern),
            format!("/{}/", trimmed_pattern),
            trimmed_pattern.to_string(),
        ];
        let is_present = variations.iter().any(|v| existing_lines.contains(v.trim()));

        if !is_present {
            final_new_patterns.push(pattern_to_check.clone());
        }
    }

    if !final_new_patterns.is_empty() || !urules_header_found {
        // Rebuild .gitignore content
        let mut new_gitignore_content = String::new();
        new_gitignore_content.push_str(&pre_content);

        // Ensure there's a newline before our section if pre_content is not empty and doesn't end with one
        if !pre_content.is_empty() && !pre_content.ends_with('\n') {
            new_gitignore_content.push('\n');
        }

        new_gitignore_content.push_str(GITIGNORE_HEADER);
        new_gitignore_content.push('\n');
        
        // Add existing lines from our old section (if any)
        new_gitignore_content.push_str(&urules_section_content); 

        // Add new patterns
        for pattern in final_new_patterns {
            if !urules_section_content.lines().any(|l| l.trim() == pattern.trim()) && 
               !pre_content.lines().any(|l| l.trim() == pattern.trim()) { // Double check not to add duplicates from pre_content
                 new_gitignore_content.push_str(&pattern);
                 new_gitignore_content.push('\n');
            }
        }
        
        new_gitignore_content.push_str(GITIGNORE_FOOTER);
        new_gitignore_content.push('\n');
        new_gitignore_content.push_str(&post_content);

        fs::write(&gitignore_path, new_gitignore_content.trim_end_matches('\n').to_string() + "\n")
            .with_context(|| format!("Failed to write updated .gitignore to {:?}", gitignore_path))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    // Assuming AgentName is pub and accessible for tests
    // If not, tests might need to be in main.rs or AgentName moved.
    use crate::AgentName; 

    fn read_gitignore_lines(gitignore_path: &Path) -> HashSet<String> {
        if !gitignore_path.exists() {
            return HashSet::new();
        }
        let file = File::open(gitignore_path).unwrap();
        let reader = BufReader::new(file);
        reader.lines().map(|l| l.unwrap().trim().to_string()).collect()
    }

    #[test]
    fn test_add_to_empty_gitignore() -> Result<()> {
        let dir = tempdir()?;
        let output_path = dir.path();
        
        update_gitignore(output_path, &AgentName::Cursor)?;
        let lines = read_gitignore_lines(&output_path.join(".gitignore"));
        
        assert!(lines.contains(GITIGNORE_HEADER));
        assert!(lines.contains(".cursor/"));
        assert!(lines.contains(GITIGNORE_FOOTER));
        Ok(())
    }

    #[test]
    fn test_add_to_existing_gitignore_no_section() -> Result<()> {
        let dir = tempdir()?;
        let output_path = dir.path();
        let gitignore_file = output_path.join(".gitignore");
        fs::write(&gitignore_file, "node_modules/\ntarget/\n")?;

        update_gitignore(output_path, &AgentName::Claude)?;
        let content = fs::read_to_string(&gitignore_file)?;
        
        assert!(content.contains("node_modules/"));
        assert!(content.contains("target/"));
        assert!(content.contains(GITIGNORE_HEADER));
        assert!(content.contains("CLAUDE.md"));
        assert!(content.contains(GITIGNORE_FOOTER));
        Ok(())
    }

    #[test]
    fn test_add_to_existing_gitignore_with_section() -> Result<()> {
        let dir = tempdir()?;
        let output_path = dir.path();
        let gitignore_file = output_path.join(".gitignore");
        let initial_content = format!("existing_pattern/\n{}\n.cursor/\n{}\nother_stuff/\n", GITIGNORE_HEADER, GITIGNORE_FOOTER);
        fs::write(&gitignore_file, initial_content)?;

        // Add Windsurf rules, which has some new patterns
        update_gitignore(output_path, &AgentName::Windsurf)?;
        let lines = read_gitignore_lines(&gitignore_file);

        assert!(lines.contains("existing_pattern/"));
        assert!(lines.contains(".cursor/")); // From original section
        assert!(lines.contains("global_rules.md")); // New
        assert!(lines.contains(".windsurf/"));      // New
        assert!(lines.contains("other_stuff/"));
        assert!(lines.contains(GITIGNORE_HEADER));
        assert!(lines.contains(GITIGNORE_FOOTER));
        
        let content = fs::read_to_string(&gitignore_file)?;
        let header_pos = content.find(GITIGNORE_HEADER).unwrap();
        let footer_pos = content.find(GITIGNORE_FOOTER).unwrap();
        let urules_section = &content[header_pos..footer_pos];
        
        assert!(urules_section.contains(".cursor/"));
        assert!(urules_section.contains("global_rules.md"));
        assert!(urules_section.contains(".windsurf/"));

        Ok(())
    }

    #[test]
    fn test_no_duplicate_patterns_added() -> Result<()> {
        let dir = tempdir()?;
        let output_path = dir.path();
        let gitignore_file = output_path.join(".gitignore");
        fs::write(&gitignore_file, ".cursor/\n")?; // Pre-existing pattern

        update_gitignore(output_path, &AgentName::Cursor)?;
        let content = fs::read_to_string(&gitignore_file)?;
        
        let occurrences = content.matches(".cursor/").count();
        assert_eq!(occurrences, 1, "Pattern '.cursor/' should only appear once.");
        assert!(content.contains(GITIGNORE_HEADER));
        assert!(content.contains(GITIGNORE_FOOTER));
        Ok(())
    }

    #[test]
    fn test_variations_of_patterns_are_detected() -> Result<()> {
        let dir = tempdir()?;
        let output_path = dir.path();
        let gitignore_file = output_path.join(".gitignore");
        // Test with variations that should prevent adding ".cursor/"
        fs::write(&gitignore_file, "/.cursor/\n")?; 
        update_gitignore(output_path, &AgentName::Cursor)?;
        let content_after_first_update = fs::read_to_string(&gitignore_file)?;
        assert_eq!(content_after_first_update.matches(".cursor/").count(), 0, "'.cursor/' should not be added if '/.cursor/' exists.");
        assert!(content_after_first_update.contains(GITIGNORE_HEADER)); // Header should be added
        
        fs::write(&gitignore_file, ".cursor\n")?; // Test another variation
        update_gitignore(output_path, &AgentName::Cursor)?;
        let content_after_second_update = fs::read_to_string(&gitignore_file)?;
        assert_eq!(content_after_second_update.matches(".cursor/").count(), 0, "'.cursor/' should not be added if '.cursor' exists.");

        Ok(())
    }
}
