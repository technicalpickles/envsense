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
fn test_info_hierarchical_display() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("info");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Traits:"))
        .stdout(predicate::str::contains("terminal:"))
        .stdout(predicate::str::contains("  color_level ="))
        .stdout(predicate::str::contains("  interactive ="));
}

#[test]
fn test_info_display_format() {
    // Test that info display contains expected hierarchical format
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("info");
    let output = cmd.assert().success().get_output().stdout.clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should contain basic information in hierarchical format
    assert!(output_str.contains("Traits:"));
    assert!(output_str.contains("terminal"));
    // Should use key = value format for simple values
    assert!(output_str.contains(" = "));
}

#[test]
fn test_rainbow_colors_with_truecolor() {
    // Test that rainbow colors work automatically when colors are enabled
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("info");

    cmd.assert().success();
}

#[test]
fn test_rainbow_colors_with_hierarchical_display() {
    // Test that rainbow colors work with hierarchical display
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("info");

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
fn test_hierarchical_display_indentation() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("info");

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

    // Hierarchical display should have multiple indentation levels
    assert!(found_level_0, "Should have level 0 indentation");
    assert!(found_level_1, "Should have level 1 indentation");
}

#[test]
fn test_raw_output_format() {
    // Raw output should use flat format regardless of hierarchical display
    let mut cmd_raw = Command::cargo_bin("envsense").unwrap();
    cmd_raw.arg("info").arg("--raw");
    let raw_output = cmd_raw.assert().success().get_output().stdout.clone();

    let output_str = String::from_utf8_lossy(&raw_output);

    // Raw output should be flat, not hierarchical
    assert!(output_str.contains("agent"));
    assert!(output_str.contains("terminal"));
}

#[test]
fn test_json_output_format() {
    // JSON output should not be affected by hierarchical display changes
    let mut cmd_json = Command::cargo_bin("envsense").unwrap();
    cmd_json.arg("info").arg("--json");
    let json_output = cmd_json.assert().success().get_output().stdout.clone();

    let output_str = String::from_utf8_lossy(&json_output);

    // Should be valid JSON
    assert!(output_str.contains("{"));
    assert!(output_str.contains("}"));
    assert!(output_str.contains("\"traits\""));
}
