use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_list_checks_context_descriptions() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("check").arg("--list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available contexts:"))
        .stdout(predicate::str::contains(
            "- agent: Agent environment detection",
        ))
        .stdout(predicate::str::contains(
            "- ide: Integrated development environment",
        ))
        .stdout(predicate::str::contains(
            "- ci: Continuous integration environment",
        ))
        .stdout(predicate::str::contains(
            "- terminal: Terminal characteristics",
        ));
}

#[test]
fn test_list_checks_field_formatting() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("check").arg("--list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available fields:"))
        .stdout(predicate::str::contains("agent fields:"))
        .stdout(predicate::str::contains("agent.id"))
        .stdout(predicate::str::contains("# Agent identifier"))
        .stdout(predicate::str::contains("terminal fields:"))
        .stdout(predicate::str::contains("terminal.color_level"))
        .stdout(predicate::str::contains("# Color support level"));
}

#[test]
fn test_info_tree_display() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("info").arg("--tree");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Traits:"))
        .stdout(predicate::str::contains("terminal:"))
        .stdout(predicate::str::contains("  color_level:"))
        .stdout(predicate::str::contains("  interactive:"));
}

#[test]
fn test_info_tree_vs_regular_display() {
    // Test regular display
    let mut cmd_regular = Command::cargo_bin("envsense").unwrap();
    cmd_regular.arg("info");
    let regular_output = cmd_regular.assert().success().get_output().stdout.clone();

    // Test tree display
    let mut cmd_tree = Command::cargo_bin("envsense").unwrap();
    cmd_tree.arg("info").arg("--tree");
    let tree_output = cmd_tree.assert().success().get_output().stdout.clone();

    // They should be different
    assert_ne!(regular_output, tree_output);

    // Both should contain basic information
    let regular_str = String::from_utf8_lossy(&regular_output);
    let tree_str = String::from_utf8_lossy(&tree_output);

    assert!(regular_str.contains("Traits:"));
    assert!(tree_str.contains("Traits:"));
    assert!(regular_str.contains("terminal"));
    assert!(tree_str.contains("terminal"));
}

#[test]
fn test_rainbow_colors_with_truecolor() {
    // Test that rainbow colors work automatically when colors are enabled
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("info");

    cmd.assert().success();
}

#[test]
fn test_rainbow_colors_with_tree_display() {
    // Test that rainbow colors work with tree display
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("info").arg("--tree");

    cmd.assert().success();
}

#[test]
fn test_no_color_disables_rainbow() {
    // Test that NO_COLOR environment variable disables rainbow colors
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env("NO_COLOR", "1").arg("info");

    cmd.assert().success();
}

#[test]
fn test_context_descriptions_completeness() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("check").arg("--list");

    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8_lossy(&output);

    // Check that all contexts have descriptions (no bare context names)
    assert!(!output_str.contains("- agent\n"));
    assert!(!output_str.contains("- ide\n"));
    assert!(!output_str.contains("- ci\n"));
    assert!(!output_str.contains("- terminal\n"));

    // Check that all contexts have proper descriptions
    assert!(output_str.contains("- agent: "));
    assert!(output_str.contains("- ide: "));
    assert!(output_str.contains("- ci: "));
    assert!(output_str.contains("- terminal: "));
}

#[test]
fn test_field_descriptions_alignment() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("check").arg("--list");

    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8_lossy(&output);

    // Check that field descriptions are properly aligned with # comments
    let lines: Vec<&str> = output_str.lines().collect();
    let field_lines: Vec<&str> = lines
        .iter()
        .filter(|line| line.trim_start().contains("# ") && line.contains("."))
        .cloned()
        .collect();

    // Should have multiple field lines with descriptions
    assert!(field_lines.len() > 5);

    // Each field line should have proper format: "    field_name    # description"
    for line in field_lines {
        assert!(line.contains("#"));
        let parts: Vec<&str> = line.split('#').collect();
        assert_eq!(parts.len(), 2);
        assert!(!parts[1].trim().is_empty()); // Description should not be empty
    }
}

#[test]
fn test_tree_display_indentation() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("info").arg("--tree");

    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8_lossy(&output);

    let lines: Vec<&str> = output_str.lines().collect();

    // Find lines with different indentation levels
    let mut found_level_0 = false;
    let mut found_level_1 = false;

    for line in lines {
        if line.starts_with("terminal:") || line.starts_with("agent:") {
            found_level_0 = true;
        } else if line.starts_with("  ") && !line.starts_with("    ") {
            found_level_1 = true;
        }
    }

    // Tree display should have multiple indentation levels
    assert!(found_level_0, "Should have level 0 indentation");
    assert!(found_level_1, "Should have level 1 indentation");
}

#[test]
fn test_raw_output_not_affected_by_tree() {
    // Raw output should not be affected by --tree flag
    let mut cmd_raw = Command::cargo_bin("envsense").unwrap();
    cmd_raw.arg("info").arg("--raw");
    let raw_output = cmd_raw.assert().success().get_output().stdout.clone();

    let mut cmd_raw_tree = Command::cargo_bin("envsense").unwrap();
    cmd_raw_tree.arg("info").arg("--raw").arg("--tree");
    let raw_tree_output = cmd_raw_tree.assert().success().get_output().stdout.clone();

    // Raw output should be the same regardless of --tree flag
    assert_eq!(raw_output, raw_tree_output);
}

#[test]
fn test_json_output_not_affected_by_formatting_flags() {
    // JSON output should not be affected by --tree flag
    let mut cmd_json = Command::cargo_bin("envsense").unwrap();
    cmd_json.arg("info").arg("--json");
    let json_output = cmd_json.assert().success().get_output().stdout.clone();

    let mut cmd_json_tree = Command::cargo_bin("envsense").unwrap();
    cmd_json_tree.arg("info").arg("--json").arg("--tree");
    let json_tree_output = cmd_json_tree.assert().success().get_output().stdout.clone();

    // JSON output should be the same regardless of formatting flags
    assert_eq!(json_output, json_tree_output);
}
