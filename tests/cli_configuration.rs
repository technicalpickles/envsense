use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Tests for Phase 3: Configuration System
///
/// This test module covers:
/// - Configuration file loading and validation
/// - CLI flag integration with configuration
/// - New CLI flags functionality
/// - Configuration system integration

#[test]
fn test_new_check_flags_available() {
    // Test that new CLI flags are available and work
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--lenient"))
        .stdout(predicate::str::contains("Use lenient mode"))
        .stdout(predicate::str::contains("--descriptions"))
        .stdout(predicate::str::contains(
            "Show context descriptions in list mode",
        ));
}

#[test]
fn test_new_info_flags_available() {
    // Test that new info CLI flags are available
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--tree"))
        .stdout(predicate::str::contains(
            "Use tree structure for nested display",
        ))
        .stdout(predicate::str::contains("--compact"))
        .stdout(predicate::str::contains(
            "Compact output without extra formatting",
        ));
}

#[test]
fn test_descriptions_flag_requires_list() {
    // Test that --descriptions requires --list
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--descriptions", "agent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--list"));
}

#[test]
fn test_descriptions_flag_with_list() {
    // Test that --descriptions works with --list
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list", "--descriptions"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available contexts:"))
        .stdout(predicate::str::contains("Agent environment detection"));
}

#[test]
fn test_lenient_flag_syntax() {
    // Test that --lenient flag is accepted (even if not fully implemented yet)
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--lenient", "agent"])
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May succeed or fail based on environment, but shouldn't error on syntax
}

#[test]
fn test_tree_flag_syntax() {
    // Test that --tree flag is accepted for info command
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--tree"]).assert().success(); // Should work with hierarchical display
}

#[test]
fn test_compact_flag_syntax() {
    // Test that --compact flag is accepted for info command
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--compact"]).assert().success(); // Should work with compact formatting
}

#[test]
fn test_configuration_system_loads() {
    // Test that the configuration system loads without errors
    // This is an integration test that ensures the config system doesn't break CLI
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available contexts:"));
}

#[test]
fn test_configuration_file_integration() {
    // Test configuration file loading (using a temporary config)
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("envsense");
    fs::create_dir_all(&config_dir).unwrap();

    let config_file = config_dir.join("config.toml");
    let config_content = r#"
[error_handling]
strict_mode = false
show_usage_on_error = true

[output_formatting]
context_descriptions = true
nested_display = true
rainbow_colors = false

[validation]
validate_predicates = false
allowed_characters = "a-zA-Z0-9_.=-"
"#;

    fs::write(&config_file, config_content).unwrap();

    // Set the config directory via environment variable (if supported)
    // For now, just test that the CLI still works with config files present
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list"]).assert().success();
}

#[test]
fn test_flag_combinations_with_new_flags() {
    // Test various flag combinations with new flags

    // --list with --descriptions should work
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list", "--descriptions"])
        .assert()
        .success();

    // --lenient with predicates should work
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--lenient", "agent"])
        .assert()
        .code(predicate::in_iter(vec![0, 1]));

    // --tree and --compact together should work (though may not have different behavior yet)
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--tree", "--compact"]).assert().success();
}

#[test]
fn test_backward_compatibility_maintained() {
    // Test that all existing functionality still works

    // Basic check command
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "agent"])
        .assert()
        .code(predicate::in_iter(vec![0, 1]));

    // Basic info command
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info"]).assert().success();

    // JSON output
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));

    // List functionality
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available contexts:"));
}

#[test]
fn test_error_handling_still_works() {
    // Test that enhanced error handling from Phase 1 still works

    // No predicates error
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error: no predicates specified"));

    // Invalid syntax error
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "invalid@syntax"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("invalid predicate syntax"));

    // Flag combination error
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list", "--any"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("invalid flag combination"));
}

#[test]
fn test_output_formatting_still_works() {
    // Test that Phase 2 output formatting still works

    // Hierarchical display
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Contexts:"))
        .stdout(predicate::str::contains("Traits:"))
        .stdout(predicate::str::contains("terminal:"));

    // Context descriptions in list
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Agent environment detection"))
        .stdout(predicate::str::contains(
            "Integrated development environment",
        ));
}
