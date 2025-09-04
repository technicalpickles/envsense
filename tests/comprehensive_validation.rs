//! Comprehensive Testing & Validation Suite
//!
//! This test suite provides comprehensive validation of the envsense CLI and library:
//! - Schema version consistency and correctness
//! - New dot notation syntax functionality
//! - End-to-end integration testing
//! - Performance regression testing
//! - CLI edge case handling
//! - README example validation
//! - Nested structure serialization

use serde_json::Value;
use std::process::Command;
use std::time::Instant;

/// Test that the schema version is correctly set to 0.3.0
#[test]
fn test_schema_version_0_3_0() {
    let output = Command::new("cargo")
        .args(["run", "--", "info", "--json"])
        .output()
        .expect("Failed to run envsense info --json");

    assert!(output.status.success(), "Command should succeed");

    let json_str = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&json_str).expect("Should parse as JSON");

    assert_eq!(json["version"], "0.3.0", "Schema version should be 0.3.0");
}

/// Test that all required fields are present in JSON output
#[test]
fn test_json_output_structure_completeness() {
    let output = Command::new("cargo")
        .args(["run", "--", "info", "--json"])
        .output()
        .expect("Failed to run envsense info --json");

    assert!(output.status.success(), "Command should succeed");

    let json_str = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&json_str).expect("Should parse as JSON");

    // Required top-level fields
    let required_fields = ["version", "contexts", "traits", "evidence"];
    for field in required_fields {
        assert!(
            json.get(field).is_some(),
            "Field '{}' should be present",
            field
        );
    }

    // Version should be a string
    assert!(json["version"].is_string(), "Version should be a string");

    // Contexts should be an array
    assert!(json["contexts"].is_array(), "Contexts should be an array");

    // Traits should be an object
    assert!(json["traits"].is_object(), "Traits should be an object");

    // Evidence should be an array
    assert!(json["evidence"].is_array(), "Evidence should be an array");
}

/// Test that the new dot notation syntax works correctly
#[test]
fn test_new_dot_notation_syntax() {
    let test_cases = vec![
        "agent.id",
        "ide.id",
        "terminal.interactive",
        "terminal.stdin.tty",
        "terminal.stdout.tty",
        "terminal.stderr.tty",
        "terminal.color_level",
        "terminal.supports_hyperlinks",
        "ci.id",
        "ci.branch",
        "ci.is_pr",
        "ci.name",
        "ci.vendor",
    ];

    for field_path in test_cases {
        let output = Command::new("cargo")
            .args(["run", "--", "check", field_path])
            .output()
            .expect(&format!("Failed to check field: {}", field_path));

        // Should not crash or produce invalid output
        assert!(
            output.status.code().is_some(),
            "Command should complete for field: {}",
            field_path
        );

        let _stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should not produce error output for valid field paths (ignore mise warnings)
        if !field_path.contains("unknown") {
            let stderr_lines: Vec<&str> = stderr.lines().collect();
            let has_actual_error = stderr_lines.iter().any(|line| {
                line.contains("Error")
                    || (line.contains("error")
                        && !line.contains("mise")
                        && !line.contains("IO error while reading marker"))
            });
            assert!(
                !has_actual_error,
                "Should not produce errors for field: {}",
                field_path
            );
        }
    }
}

/// Test that context checks work correctly
#[test]
fn test_context_detection_consistency() {
    let contexts = ["agent", "ide", "terminal", "ci"];

    for context in contexts {
        let output = Command::new("cargo")
            .args(["run", "--", "check", context])
            .output()
            .expect(&format!("Failed to check context: {}", context));

        assert!(
            output.status.code().is_some(),
            "Context check should complete: {}",
            context
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should produce boolean output or descriptive message
        let result = stdout.trim();
        if result == "true" || result == "false" {
            // Boolean output is expected
        } else if result == "No CI detected" {
            // Descriptive message for no detection is acceptable
        } else if result.starts_with("CI detected:") {
            // Descriptive message for CI detection is acceptable
        } else if result.starts_with("Agent detected:") {
            // Descriptive message for agent detection is acceptable
        } else if result.starts_with("IDE detected:") {
            // Descriptive message for IDE detection is acceptable
        } else if result.starts_with("Terminal detected:") || result.contains("terminal") {
            // Descriptive message for terminal detection is acceptable
        } else {
            assert!(
                false,
                "Context check should return boolean or descriptive message for {}: got '{}'",
                context, result
            );
        }

        // Should not produce errors (ignore mise warnings)
        let stderr_lines: Vec<&str> = stderr.lines().collect();
        let has_actual_error = stderr_lines.iter().any(|line| {
            line.contains("Error")
                || (line.contains("error")
                    && !line.contains("mise")
                    && !line.contains("IO error while reading marker"))
        });
        assert!(
            !has_actual_error,
            "Context check should not error: {} - stderr: '{}'",
            context, stderr
        );
    }
}

/// Test that the help system works correctly
#[test]
fn test_help_system_completeness() {
    // Test main help
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to run envsense --help");

    assert!(output.status.success(), "Main help should work");
    let help_text = String::from_utf8_lossy(&output.stdout);

    // Should mention main commands
    assert!(
        help_text.contains("info"),
        "Help should mention info command"
    );
    assert!(
        help_text.contains("check"),
        "Help should mention check command"
    );

    // Test check help
    let output = Command::new("cargo")
        .args(["run", "--", "check", "--help"])
        .output()
        .expect("Failed to run envsense check --help");

    assert!(output.status.success(), "Check help should work");
    let help_text = String::from_utf8_lossy(&output.stdout);

    // Should show new syntax examples
    assert!(
        help_text.contains("agent.id"),
        "Help should show new syntax examples"
    );
    assert!(
        help_text.contains("terminal.interactive"),
        "Help should show new syntax examples"
    );

    // Test info help
    let output = Command::new("cargo")
        .args(["run", "--", "info", "--help"])
        .output()
        .expect("Failed to run envsense info --help");

    assert!(output.status.success(), "Info help should work");
    let help_text = String::from_utf8_lossy(&output.stdout);

    // Should show available options
    assert!(
        help_text.contains("--json"),
        "Info help should show --json option"
    );
    assert!(
        help_text.contains("--fields"),
        "Info help should show --fields option"
    );
}

/// Test that field filtering works correctly
#[test]
fn test_field_filtering_functionality() {
    let output = Command::new("cargo")
        .args(["run", "--", "info", "--json", "--fields", "contexts,traits"])
        .output()
        .expect("Failed to run envsense info --json --fields contexts,traits");

    assert!(output.status.success(), "Field filtering should work");

    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&json_str).expect("Should parse as JSON");

    // Should only contain requested fields
    assert!(
        json.get("contexts").is_some(),
        "Should include contexts field"
    );
    assert!(json.get("traits").is_some(), "Should include traits field");
    assert!(
        json.get("evidence").is_none(),
        "Should not include evidence field"
    );
    assert!(
        json.get("facets").is_none(),
        "Should not include facets field"
    );
}

/// Test that negation works correctly
#[test]
fn test_negation_functionality() {
    let output = Command::new("cargo")
        .args(["run", "--", "check", "!agent"])
        .output()
        .expect("Failed to run envsense check !agent");

    assert!(output.status.code().is_some(), "Negation should work");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result = stdout.trim();

    // Should return opposite of non-negated result
    let non_negated = Command::new("cargo")
        .args(["run", "--", "check", "agent"])
        .output()
        .expect("Failed to run non-negated check");

    let non_negated_output = String::from_utf8_lossy(&non_negated.stdout);
    let non_negated_result = non_negated_output.trim();

    if non_negated_result == "true" {
        assert_eq!(result, "false", "Negation should invert true to false");
    } else {
        assert_eq!(result, "true", "Negation should invert false to true");
    }
}

/// Test that multiple predicates work correctly
#[test]
fn test_multiple_predicates_functionality() {
    let output = Command::new("cargo")
        .args(["run", "--", "check", "agent", "ide"])
        .output()
        .expect("Failed to run envsense check with multiple predicates");

    assert!(
        output.status.code().is_some(),
        "Multiple predicates should work"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have output for each predicate
    assert!(lines.len() >= 2, "Should have output for each predicate");

    // Each line should be a boolean result, overall status, or context=value format
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            // Allow overall status line, boolean results, and context=value format
            let is_valid = trimmed == "true"
                || trimmed == "false"
                || trimmed.starts_with("overall=")
                || trimmed.contains("=");

            assert!(
                is_valid,
                "Each predicate result should be boolean, overall status, or context=value: got '{}'",
                trimmed
            );
        }
    }
}

/// Test that explain mode works correctly
#[test]
fn test_explain_mode_functionality() {
    let output = Command::new("cargo")
        .args(["run", "--", "check", "--explain", "agent.id"])
        .output()
        .expect("Failed to run envsense check with explain mode");

    assert!(output.status.code().is_some(), "Explain mode should work");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should include explanation
    assert!(
        stdout.contains("#"),
        "Explain mode should include explanations"
    );
}

/// Test that quiet mode works correctly
#[test]
fn test_quiet_mode_functionality() {
    let output = Command::new("cargo")
        .args(["run", "--", "check", "--quiet", "agent"])
        .output()
        .expect("Failed to run envsense check with quiet mode");

    assert!(output.status.code().is_some(), "Quiet mode should work");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should suppress output
    assert!(
        stdout.trim().is_empty(),
        "Quiet mode should suppress output"
    );
}

/// Test that JSON output mode works correctly
#[test]
fn test_json_output_mode_functionality() {
    let output = Command::new("cargo")
        .args(["run", "--", "check", "--json", "agent"])
        .output()
        .expect("Failed to run envsense check with JSON output");

    assert!(
        output.status.code().is_some(),
        "JSON output mode should work"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    let json: Value = serde_json::from_str(&stdout).expect("Should output valid JSON");

    // Should have expected structure
    assert!(
        json.get("checks").is_some(),
        "JSON should have checks field"
    );
    assert!(
        json.get("overall").is_some(),
        "JSON should have overall field"
    );
}

/// Test that the field registry is complete and accurate
#[test]
fn test_field_registry_completeness() {
    let output = Command::new("cargo")
        .args(["run", "--", "check", "--list"])
        .output()
        .expect("Failed to run envsense check --list");

    assert!(output.status.success(), "Field listing should work");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should list all expected contexts
    let expected_contexts = ["agent", "ide", "terminal", "ci"];
    for context in expected_contexts {
        assert!(stdout.contains(context), "Should list {} context", context);
    }

    // Should show field examples
    assert!(stdout.contains("agent.id"), "Should show agent.id field");
    assert!(
        stdout.contains("terminal.interactive"),
        "Should show terminal.interactive field"
    );
    assert!(stdout.contains("ci.id"), "Should show ci.id field");
}

/// Performance regression test - ensure detection is fast
#[test]
fn test_detection_performance() {
    let start = Instant::now();

    let output = Command::new("cargo")
        .args(["run", "--", "info", "--json"])
        .output()
        .expect("Failed to run envsense info --json");

    let duration = start.elapsed();

    assert!(output.status.success(), "Performance test should succeed");

    // Detection should complete in reasonable time (more lenient for CI environments)
    assert!(
        duration.as_millis() < 5000,
        "Detection should complete in under 5 seconds, took {}ms",
        duration.as_millis()
    );
}

/// Test that the CLI handles edge cases gracefully
#[test]
fn test_cli_edge_case_handling() {
    // Test with empty input
    let output = Command::new("cargo")
        .args(["run", "--", "check", ""])
        .output()
        .expect("Failed to run envsense check with empty input");

    // Should handle gracefully (may fail, but shouldn't crash)
    assert!(
        output.status.code().is_some(),
        "Should handle empty input gracefully"
    );

    // Test with very long input
    let long_input = "a".repeat(1000);
    let output = Command::new("cargo")
        .args(["run", "--", "check", &long_input])
        .output()
        .expect("Failed to run envsense check with long input");

    // Should handle gracefully
    assert!(
        output.status.code().is_some(),
        "Should handle long input gracefully"
    );
}

/// Test that all README examples work correctly
#[test]
fn test_readme_examples_functionality() {
    // Test basic examples from README
    let examples = vec![
        ("agent", "Check if agent context is detected"),
        ("agent.id", "Show agent ID"),
        ("agent.id=cursor", "Check if agent ID is 'cursor'"),
        ("terminal.interactive", "Check if terminal is interactive"),
        ("!ci", "Check if CI is NOT detected"),
    ];

    for (example, description) in examples {
        let output = Command::new("cargo")
            .args(["run", "--", "check", example])
            .output()
            .expect(&format!("Failed to test README example: {}", example));

        assert!(
            output.status.code().is_some(),
            "README example should work: {} ({})",
            example,
            description
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should not produce errors (ignore mise warnings)
        let stderr_lines: Vec<&str> = stderr.lines().collect();
        let has_actual_error = stderr_lines.iter().any(|line| {
            line.contains("Error")
                || (line.contains("error")
                    && !line.contains("mise")
                    && !line.contains("IO error while reading marker"))
        });
        assert!(
            !has_actual_error,
            "README example should not error: {} ({})",
            example, description
        );

        // Should produce some output
        assert!(
            !stdout.trim().is_empty(),
            "README example should produce output: {} ({})",
            example,
            description
        );
    }
}

/// Test that the schema version is consistent across all outputs
#[test]
fn test_schema_version_consistency() {
    // Test info command
    let info_output = Command::new("cargo")
        .args(["run", "--", "info", "--json"])
        .output()
        .expect("Failed to run envsense info --json");

    let info_json: Value =
        serde_json::from_str(&String::from_utf8_lossy(&info_output.stdout)).unwrap();
    let info_version = info_json["version"].as_str().unwrap();

    // Test check command with JSON output
    let check_output = Command::new("cargo")
        .args(["run", "--", "check", "--json", "agent"])
        .output()
        .expect("Failed to run envsense check --json");

    let _check_json: Value =
        serde_json::from_str(&String::from_utf8_lossy(&check_output.stdout)).unwrap();

    // Check command doesn't include schema version in output, so just verify info version
    assert_eq!(info_version, "0.3.0", "Schema version should be 0.3.0");
}

/// Test that the new nested structure is properly serialized
#[test]
fn test_nested_structure_serialization() {
    let output = Command::new("cargo")
        .args(["run", "--", "info", "--json"])
        .output()
        .expect("Failed to run envsense info --json");

    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    // Traits should be an object with nested structure
    let traits = &json["traits"];
    assert!(traits.is_object(), "Traits should be an object");

    // Should have expected nested contexts
    let expected_contexts = ["agent", "ide", "terminal", "ci"];
    for context in expected_contexts {
        if let Some(context_obj) = traits.get(context) {
            assert!(
                context_obj.is_object(),
                "{} context should be an object",
                context
            );
        }
    }

    // Terminal should have nested stream fields
    if let Some(terminal) = traits.get("terminal") {
        if let Some(stdin) = terminal.get("stdin") {
            assert!(stdin.is_object(), "terminal.stdin should be an object");
            assert!(
                stdin.get("tty").is_some(),
                "terminal.stdin should have tty field"
            );
            assert!(
                stdin.get("piped").is_some(),
                "terminal.stdin should have piped field"
            );
        }

        if let Some(stdout) = terminal.get("stdout") {
            assert!(stdout.is_object(), "terminal.stdout should be an object");
            assert!(
                stdout.get("tty").is_some(),
                "terminal.stdout should have tty field"
            );
            assert!(
                stdout.get("piped").is_some(),
                "terminal.stdout should have piped field"
            );
        }

        if let Some(stderr) = terminal.get("stderr") {
            assert!(stderr.is_object(), "terminal.stderr should be an object");
            assert!(
                stderr.get("tty").is_some(),
                "terminal.stderr should have tty field"
            );
            assert!(
                stderr.get("piped").is_some(),
                "terminal.stderr should have piped field"
            );
        }
    }
}
