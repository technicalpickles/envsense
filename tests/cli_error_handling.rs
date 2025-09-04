use assert_cmd::Command;
use predicates::prelude::*;

/// Tests for Phase 1: Enhanced Error Handling
///
/// This test module covers:
/// - Improved check command usage errors
/// - Flag combination validation
/// - Predicate syntax validation
/// - Field path validation

#[test]
fn test_no_predicates_specified_error() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error: no predicates specified"))
        .stderr(predicate::str::contains(
            "Usage: envsense check <predicate> [<predicate>...]",
        ))
        .stderr(predicate::str::contains("Examples:"))
        .stderr(predicate::str::contains("envsense check agent"))
        .stderr(predicate::str::contains("envsense check ide.cursor"))
        .stderr(predicate::str::contains("envsense check --list"));
}

#[test]
fn test_flag_validation_list_with_any() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list", "--any"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error: invalid flag combination: --list cannot be used with --any or --all"))
        .stderr(predicate::str::contains("The --list flag shows available predicates, while --any/--all control evaluation logic"))
        .stderr(predicate::str::contains("Usage examples:"))
        .stderr(predicate::str::contains("envsense check --list"))
        .stderr(predicate::str::contains("envsense check --any agent ide"));
}

#[test]
fn test_flag_validation_list_with_all() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list", "--all"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Error: invalid flag combination: --list cannot be used with --any or --all",
        ));
}

#[test]
fn test_flag_validation_any_with_all() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--any", "--all", "agent"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Error: invalid flag combination: --any and --all cannot be used together",
        ))
        .stderr(predicate::str::contains(
            "These flags control different evaluation modes and are mutually exclusive",
        ))
        .stderr(predicate::str::contains(
            "--any: succeeds if ANY predicate matches",
        ))
        .stderr(predicate::str::contains(
            "--all: succeeds if ALL predicates match (default behavior)",
        ));
}

#[test]
fn test_flag_validation_list_with_predicates() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list", "agent"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error: invalid flag combination: --list cannot be used with predicates"))
        .stderr(predicate::str::contains("The --list flag shows all available predicates, so providing specific predicates is redundant"))
        .stderr(predicate::str::contains("envsense check --list"))
        .stderr(predicate::str::contains("envsense check agent"));
}

#[test]
fn test_flag_validation_list_with_quiet() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list", "--quiet"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Error: invalid flag combination: --list cannot be used with --quiet",
        ))
        .stderr(predicate::str::contains(
            "The --list flag is designed to show information, while --quiet suppresses output",
        ))
        .stderr(predicate::str::contains("envsense check --list"))
        .stderr(predicate::str::contains("envsense check agent --quiet"));
}

#[test]
fn test_predicate_syntax_validation_invalid_characters() {
    let test_cases = vec![
        ("invalid@syntax", "@"),
        ("agent#test", "#"),
        ("ide$cursor", "$"),
        ("test%value", "%"),
        ("bad&predicate", "&"),
        ("invalid*field", "*"),
        ("test+value", "+"),
        ("bad-predicate", "-"),
        ("test(value)", "("),
        ("test[value]", "["),
        ("test{value}", "{"),
        ("test|value", "|"),
        ("test\\value", "\\"),
        ("test/value", "/"),
        ("test:value", ":"),
        ("test;value", ";"),
        ("test<value", "<"),
        ("test>value", ">"),
        ("test?value", "?"),
        ("test\"value", "\""),
        ("test'value", "'"),
        ("test value", " "),
        ("test\tvalue", "\t"),
        ("test\nvalue", "\n"),
    ];

    for (invalid_predicate, _invalid_char) in test_cases {
        let mut cmd = Command::cargo_bin("envsense").unwrap();
        cmd.args(["check", invalid_predicate])
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains(format!("Error parsing '{}': invalid predicate syntax", invalid_predicate)))
            .stderr(predicate::str::contains("Valid predicate syntax: letters, numbers, dots (.), equals (=), and underscores (_) only"));
    }
}

#[test]
fn test_predicate_syntax_validation_valid_characters() {
    let valid_cases = vec![
        "agent",
        "agent.id",
        "agent.id=cursor",
        "ide.cursor",
        "ci.github",
        "terminal.interactive",
        "agent_test",
        "test_field.sub_field",
        "field123",
        "test123.field456",
        "a.b.c",
        "field=value123",
        "field_name=test_value",
        "!agent",
        "!agent.id=test",
    ];

    for valid_predicate in valid_cases {
        let mut cmd = Command::cargo_bin("envsense").unwrap();
        // These might fail for other reasons (like field not found), but should not fail on syntax
        let output = cmd.args(["check", valid_predicate]).output().unwrap();

        // Check that it's not a syntax error (exit code 2 with syntax error message)
        if output.status.code() == Some(2) {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !stderr.contains("invalid predicate syntax"),
                "Valid predicate '{}' failed syntax validation: {}",
                valid_predicate,
                stderr
            );
        }
    }
}

#[test]
fn test_field_path_validation_invalid_field() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "agent.invalid_field"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Error: invalid field path 'agent.invalid_field': available fields for 'agent': agent.id"));
}

#[test]
fn test_field_path_validation_unknown_context() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "unknown.field"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "Error parsing 'unknown.field': invalid field path",
        ));
}

#[test]
fn test_field_path_validation_multiple_invalid_fields() {
    let invalid_fields = vec![
        (
            "agent.nonexistent",
            "available fields for 'agent': agent.id",
        ),
        ("ide.invalid", "available fields for 'ide': ide.id"),
        ("ci.fake", "available fields for 'ci'"),
        ("terminal.bogus", "available fields for 'terminal'"),
    ];

    for (invalid_field, expected_message) in invalid_fields {
        let mut cmd = Command::cargo_bin("envsense").unwrap();
        cmd.args(["check", invalid_field])
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains(format!(
                "Error: invalid field path '{}'",
                invalid_field
            )))
            .stderr(predicate::str::contains(expected_message));
    }
}

#[test]
fn test_empty_predicate_syntax_validation() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", ""])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Error parsing '': empty input"));
}

#[test]
fn test_whitespace_only_predicate() {
    let whitespace_cases = vec!["   ", "\t", "\n", "\r", "  \t  \n  "];

    for whitespace in whitespace_cases {
        let mut cmd = Command::cargo_bin("envsense").unwrap();
        cmd.args(["check", whitespace])
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("Error parsing"))
            .stderr(predicate::str::contains("empty input"));
    }
}

#[test]
fn test_negated_invalid_syntax() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "!invalid@syntax"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Error parsing '!invalid@syntax': invalid predicate syntax"))
        .stderr(predicate::str::contains("Valid predicate syntax: letters, numbers, dots (.), equals (=), and underscores (_) only"));
}

#[test]
fn test_negated_invalid_field() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "!agent.invalid_field"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Error: invalid field path 'agent.invalid_field': available fields for 'agent': agent.id"));
}

#[test]
fn test_multiple_predicates_with_invalid_syntax() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "agent", "invalid@syntax"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "Error parsing 'invalid@syntax': invalid predicate syntax",
        ));
}

#[test]
fn test_multiple_predicates_with_invalid_field() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "agent", "agent.invalid_field"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "Error: invalid field path 'agent.invalid_field'",
        ));
}

#[test]
fn test_flag_combinations_that_should_work() {
    // Test valid flag combinations that should not trigger validation errors

    // --list by itself should work
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("contexts:"))
        .stdout(predicate::str::contains("fields:"));

    // --any with predicates should work
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--any", "agent", "ide"])
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May succeed or fail based on environment

    // --all with predicates should work
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--all", "agent", "ide"])
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May succeed or fail based on environment

    // --quiet with predicates should work
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--quiet", "agent"])
        .assert()
        .code(predicate::in_iter(vec![0, 1]))
        .stdout(predicate::str::is_empty()); // Should be quiet

    // --explain with predicates should work
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--explain", "agent"])
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May succeed or fail based on environment
}

#[test]
fn test_error_message_formatting() {
    // Test that error messages are well-formatted and helpful

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    let output = cmd.args(["check"]).output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should have proper structure
    assert!(stderr.contains("Error: no predicates specified"));
    assert!(stderr.contains("Usage: envsense check"));
    assert!(stderr.contains("Examples:"));
    assert!(stderr.contains("For more information, see: envsense check --help"));

    // Should have proper line breaks
    assert!(stderr.lines().count() >= 8); // Multiple lines of helpful information
}

#[test]
fn test_exit_codes() {
    // Test that different error conditions return appropriate exit codes

    // No predicates: exit code 1
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check"]).assert().failure().code(1);

    // Invalid flag combination: exit code 1
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list", "--any"])
        .assert()
        .failure()
        .code(1);

    // Invalid syntax: exit code 2
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "invalid@syntax"])
        .assert()
        .failure()
        .code(2);

    // Invalid field path: exit code 2
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "agent.invalid"])
        .assert()
        .failure()
        .code(2);
}
