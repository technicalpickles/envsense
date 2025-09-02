use assert_cmd::Command;
use predicates::str::contains;
use serde_json::Value;

/// Integration tests for the new output formatting system (Task 2.4)
/// These tests verify the complete CLI output formatting functionality

#[test]
fn cli_new_dot_notation_syntax() {
    // Test basic dot notation field access
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent.id"])
        .assert()
        .success()
        .stdout("cursor\n");

    // Test dot notation field comparison
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent.id=cursor"])
        .assert()
        .success()
        .stdout("true\n");

    // Test dot notation field comparison failure
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent.id=other"])
        .assert()
        .failure()
        .stdout("false\n");
}

#[test]
fn cli_explain_mode_with_different_result_types() {
    // Test explain mode with boolean result (context check)
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--explain", "agent"])
        .assert()
        .success()
        .stdout(contains("true  # reason: context 'agent' detected"));

    // Test explain mode with string result (field value)
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--explain", "agent.id"])
        .assert()
        .success()
        .stdout(contains("cursor  # reason: field value: agent.id"));

    // Test explain mode with comparison result
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--explain", "agent.id=cursor"])
        .assert()
        .success()
        .stdout(contains(
            "true  # reason: field comparison: agent.id == cursor",
        ));
}

#[test]
fn cli_json_output_with_new_system() {
    // Test JSON output structure with single predicate
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--json", "agent.id=cursor"])
        .assert()
        .success()
        .stdout(contains("\"overall\":true"))
        .stdout(contains("\"mode\":\"all\""))
        .stdout(contains("\"checks\":["))
        .stdout(contains("\"predicate\":\"agent.id=cursor\""))
        .stdout(contains("\"result\":true"));

    // Test JSON output with explain mode
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--json", "--explain", "agent.id=cursor"])
        .assert()
        .success()
        .stdout(contains("field comparison: agent.id == cursor"));

    // Verify JSON is valid by parsing it
    let output = Command::cargo_bin("envsense")
        .unwrap()
        .env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--json", "agent.id=cursor"])
        .output()
        .unwrap();

    let json_str = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&json_str).expect("Should be valid JSON");

    assert_eq!(json["overall"].as_bool().unwrap(), true);
    assert_eq!(json["mode"].as_str().unwrap(), "all");
    assert!(json["checks"].is_array());
    assert_eq!(json["checks"].as_array().unwrap().len(), 1);
}

#[test]
fn cli_multiple_predicates_with_mixed_results() {
    // Test multiple predicates with different result types
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent", "agent.id"])
        .assert()
        .success()
        .stdout(contains("overall=true"))
        .stdout(contains("agent=true"))
        .stdout(contains("agent.id=cursor"));

    // Test multiple predicates with explain mode
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--explain", "agent", "agent.id=cursor"])
        .assert()
        .success()
        .stdout(contains("overall=true"))
        .stdout(contains("agent=true  # reason: context 'agent' detected"))
        .stdout(contains(
            "agent.id=cursor=true  # reason: field comparison: agent.id == cursor",
        ));
}

#[test]
fn cli_any_vs_all_mode_output() {
    // Test --any mode with mixed results
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--any", "agent", "nonexistent"])
        .assert()
        .success()
        .stdout(contains("overall=true"));

    // Test --all mode with mixed results (should fail)
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--all", "agent", "nonexistent"])
        .assert()
        .failure()
        .stdout(contains("overall=false"));

    // Test JSON output shows correct mode
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--json", "--any", "agent", "nonexistent"])
        .assert()
        .success()
        .stdout(contains("\"mode\":\"any\""));
}

#[test]
fn cli_negation_with_new_output_system() {
    // Test negation with boolean result
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "!agent"])
        .assert()
        .success()
        .stdout("true\n");

    // Test negation with explain mode
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "--explain", "!agent"])
        .assert()
        .success()
        .stdout(contains(
            "true  # reason: negated: context 'agent' not detected",
        ));

    // Test negation with comparison
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "!agent.id=other"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn cli_legacy_syntax_compatibility() {
    // Test that legacy syntax still works with new output system
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "facet:agent_id=cursor"])
        .assert()
        .success()
        .stdout("true\n");

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "trait:is_interactive"])
        .assert(); // May succeed or fail depending on environment

    // Test legacy syntax with explain mode
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--explain", "facet:agent_id=cursor"])
        .assert()
        .success()
        .stdout(contains(
            "true  # reason: legacy facet comparison: agent_id=cursor",
        ));
}

#[test]
fn cli_terminal_field_access() {
    // Test terminal field access (these should work regardless of actual terminal state)
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "terminal.interactive"])
        .assert(); // Will be true or false, but should not error

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "terminal.stdin.tty"])
        .assert(); // Will be true or false, but should not error

    // Test with comparison
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "terminal.interactive=false"])
        .assert(); // May succeed or fail depending on actual state

    // Test with explain mode
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "--explain", "terminal.interactive"])
        .assert()
        .stdout(contains("# reason: field value: terminal.interactive"));
}

#[test]
fn cli_error_handling_with_new_system() {
    // Test invalid field path
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "invalid.field"])
        .assert()
        .failure()
        .stderr(contains("invalid check expression"));

    // Test malformed syntax
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "facet:"])
        .assert()
        .failure()
        .stderr(contains("invalid check expression"));

    // Test empty predicate
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", ""])
        .assert()
        .failure()
        .stderr(contains("invalid check expression"));
}

#[test]
fn cli_special_characters_in_values() {
    // Test with CI environment that has special characters
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("GITHUB_ACTIONS", "true")
        .env("GITHUB_REF_NAME", "feature/test-123")
        .args(["check", "ci.branch"])
        .assert(); // Should work but may show different values

    // Test comparison with special characters (using a known CI field)
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("GITHUB_ACTIONS", "true")
        .args(["check", "ci.vendor=github_actions"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn cli_json_pretty_printing() {
    // Test that JSON is pretty-printed with --explain
    let output = Command::cargo_bin("envsense")
        .unwrap()
        .env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--json", "--explain", "agent"])
        .output()
        .unwrap();

    let json_str = String::from_utf8(output.stdout).unwrap();

    // Pretty-printed JSON should have newlines and indentation
    assert!(json_str.contains("{\n"));
    assert!(json_str.contains("  \""));

    // Should be valid JSON
    let _json: Value = serde_json::from_str(&json_str).expect("Should be valid JSON");

    // Test that regular JSON (without --explain) is compact
    let output_compact = Command::cargo_bin("envsense")
        .unwrap()
        .env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--json", "agent"])
        .output()
        .unwrap();

    let compact_json = String::from_utf8(output_compact.stdout).unwrap();

    // Compact JSON should not have extra whitespace
    assert!(!compact_json.contains("{\n"));
    assert!(compact_json.len() < json_str.len()); // Should be shorter
}
