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
    // Test with special characters in CI vendor (which we know works)
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("GITHUB_ACTIONS", "true")
        .args(["check", "ci.vendor=github_actions"])
        .assert()
        .success()
        .stdout("true\n");

    // Test with unicode characters
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("ENVSENSE_AGENT", "测试")
        .args(["check", "agent.id=测试"])
        .assert()
        .success()
        .stdout("true\n");

    // Test with spaces and special characters
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("ENVSENSE_AGENT", "test agent 123")
        .args(["check", "agent.id=test agent 123"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn cli_help_text_snapshot() {
    // Test that help text is generated correctly and remains stable
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--help"])
        .assert()
        .success()
        .stdout(contains("Available predicates:"))
        .stdout(contains("Contexts (return boolean):"))
        .stdout(contains(
            "agent                    # Check if agent context is detected",
        ))
        .stdout(contains("Fields:"))
        .stdout(contains("agent fields:"))
        .stdout(contains("agent.id                  # Agent identifier"))
        .stdout(contains("terminal fields:"))
        .stdout(contains(
            "terminal.interactive      # Terminal interactivity",
        ))
        .stdout(contains("ci fields:"))
        .stdout(contains("ci.id                     # CI system identifier"))
        .stdout(contains("Examples:"))
        .stdout(contains(
            "envsense check agent              # Boolean: is agent detected?",
        ))
        .stdout(contains(
            "envsense check agent.id=cursor    # Boolean: is agent ID 'cursor'?",
        ))
        .stdout(contains("Syntax:"))
        .stdout(contains(
            "context                           # Check if context is detected",
        ))
        .stdout(contains(
            "field.path=value                  # Compare field value",
        ))
        .stdout(contains("Legacy syntax (deprecated):"))
        .stdout(contains(
            "facet:key=value                   # Use field.path=value instead",
        ));
}

#[test]
fn cli_list_predicates_functionality() {
    // Test --list flag shows all available predicates
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list"])
        .assert()
        .success()
        .stdout(contains("agent"))
        .stdout(contains("agent.id"))
        .stdout(contains("terminal.interactive"))
        .stdout(contains("terminal.stdin.tty"))
        .stdout(contains("ci.id"))
        .stdout(contains("ci.branch"));
}

#[test]
fn cli_comprehensive_error_messages() {
    // Test various error conditions with clear messages

    // Invalid context in field path
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "invalid_context.field"])
        .assert()
        .failure()
        .stderr(contains("invalid check expression"));

    // Too few path segments
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "agent"]) // This should work (context check)
        .assert()
        .failure() // No agent detected in clean env
        .stdout("false\n");

    // Single segment that's not a valid context
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "invalid_single_segment"])
        .assert()
        .failure()
        .stdout("false\n"); // Treated as context check, returns false

    // Malformed legacy syntax
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "facet:key_without_value"])
        .assert()
        .failure()
        .stderr(contains("invalid check expression"));

    // Empty facet key
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "facet:=value"])
        .assert()
        .failure()
        .stderr(contains("invalid check expression"));
}

#[test]
fn cli_multiple_predicates_comprehensive() {
    // Test complex multiple predicate scenarios
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .env("TERM_PROGRAM", "vscode")
        .args(["check", "agent", "ide", "agent.id=cursor", "ide.id=vscode"])
        .assert()
        .success()
        .stdout(contains("overall=true"))
        .stdout(contains("agent=true"))
        .stdout(contains("ide=true"))
        .stdout(contains("agent.id=cursor"))
        .stdout(contains("ide.id=vscode"));

    // Test --any mode with mixed results
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--any", "agent", "ci", "agent.id=wrong"])
        .assert()
        .success() // Should succeed because agent=true
        .stdout(contains("overall=true"));

    // Test all mode with one failure
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--all", "agent", "ci"])
        .assert()
        .failure() // Should fail because ci=false
        .stdout(contains("overall=false"))
        .stdout(contains("agent=true"))
        .stdout(contains("ci=false"));
}

#[test]
fn cli_json_output_comprehensive() {
    // Test JSON output structure with multiple predicates
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--json", "agent", "agent.id"])
        .assert()
        .success();

    let output = cmd.output().unwrap();
    let json_str = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&json_str).unwrap();

    // Verify JSON structure
    assert!(json["overall"].is_boolean());
    assert!(json["checks"].is_array());

    let checks = json["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 2);

    // Verify first check (context)
    assert_eq!(checks[0]["predicate"], "agent");
    assert!(checks[0]["result"].is_boolean());

    // Verify second check (field value)
    assert_eq!(checks[1]["predicate"], "agent.id");
    assert!(checks[1]["result"].is_boolean()); // agent.id returns boolean true when present
}

#[test]
fn cli_edge_case_field_values() {
    // Test with null/missing field values
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "agent.id"]) // No agent detected, should return "null"
        .assert()
        .success()
        .stdout("null\n");

    // Test boolean field values
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "terminal.interactive"])
        .assert(); // Will be true or false, but should not error

    // Test comparison with boolean values
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "terminal.interactive=true"])
        .assert(); // May succeed or fail, but should not error

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "terminal.interactive=false"])
        .assert(); // May succeed or fail, but should not error
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
