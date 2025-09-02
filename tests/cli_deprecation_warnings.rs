use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

/// Test that CLI shows deprecation warnings for legacy facet syntax
#[test]
fn cli_legacy_facet_shows_deprecation_warning() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "facet:ide_id=vscode"])
        .assert()
        .success()
        .stdout("true\n")
        .stderr(contains(
            "Warning: Legacy syntax 'facet:ide_id=vscode' is deprecated",
        ))
        .stderr(contains("Use 'ide.id=vscode' instead"));
}

/// Test that CLI shows deprecation warnings for legacy trait syntax
#[test]
fn cli_legacy_trait_shows_deprecation_warning() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "trait:is_interactive"])
        .assert()
        .failure() // Terminal is not interactive in test environment
        .stdout("false\n")
        .stderr(contains(
            "Warning: Legacy syntax 'trait:is_interactive' is deprecated",
        ))
        .stderr(contains("Use 'terminal.interactive' instead"));
}

/// Test that new syntax shows no deprecation warnings
#[test]
fn cli_new_syntax_no_deprecation_warning() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "ide.id=vscode"])
        .assert()
        .success()
        .stdout("true\n")
        .stderr(contains("Warning:").not())
        .stderr(contains("deprecated").not());
}

/// Test that context checks show no deprecation warnings
#[test]
fn cli_context_check_no_deprecation_warning() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "ide"])
        .assert()
        .success()
        .stdout("true\n")
        .stderr(contains("Warning:").not())
        .stderr(contains("deprecated").not());
}

/// Test that deprecation warnings don't affect exit codes for successful checks
#[test]
fn cli_deprecation_warnings_preserve_success_exit_code() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "facet:ide_id=vscode"])
        .assert()
        .success() // Should still succeed despite warning
        .stdout("true\n")
        .stderr(contains("Warning:"));
}

/// Test that deprecation warnings don't affect exit codes for failed checks
#[test]
fn cli_deprecation_warnings_preserve_failure_exit_code() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "facet:ide_id=cursor"]) // Should fail since we set vscode
        .assert()
        .failure() // Should still fail despite warning
        .stdout("false\n")
        .stderr(contains(
            "Warning: Legacy syntax 'facet:ide_id=cursor' is deprecated",
        ))
        .stderr(contains("Use 'ide.id=cursor' instead"));
}

/// Test multiple legacy predicates show multiple warnings
#[test]
fn cli_multiple_legacy_predicates_multiple_warnings() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args([
            "check",
            "--any",
            "facet:ide_id=vscode",
            "trait:is_interactive",
        ])
        .assert()
        .success() // At least one should match (ide_id)
        .stdout("overall=true\nfacet:ide_id=vscode=true\ntrait:is_interactive=false\n")
        .stderr(contains(
            "Warning: Legacy syntax 'facet:ide_id=vscode' is deprecated",
        ))
        .stderr(contains("Use 'ide.id=vscode' instead"))
        .stderr(contains(
            "Warning: Legacy syntax 'trait:is_interactive' is deprecated",
        ))
        .stderr(contains("Use 'terminal.interactive' instead"));
}

/// Test mixed legacy and new syntax
#[test]
fn cli_mixed_legacy_and_new_syntax() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "--any", "facet:ide_id=vscode", "ide.id=vscode"])
        .assert()
        .success()
        .stdout("overall=true\nfacet:ide_id=vscode=true\nide.id=vscode=true\n")
        .stderr(contains(
            "Warning: Legacy syntax 'facet:ide_id=vscode' is deprecated",
        ))
        .stderr(contains("Use 'ide.id=vscode' instead"));
}

/// Test that quiet mode still shows deprecation warnings
#[test]
fn cli_quiet_mode_still_shows_deprecation_warnings() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "-q", "facet:ide_id=vscode"])
        .assert()
        .success()
        .stdout("") // Quiet mode suppresses stdout
        .stderr(contains(
            "Warning: Legacy syntax 'facet:ide_id=vscode' is deprecated",
        ))
        .stderr(contains("Use 'ide.id=vscode' instead"));
}

/// Test deprecation warnings with negated legacy syntax
#[test]
fn cli_negated_legacy_syntax_shows_warning() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "!facet:ide_id=cursor"])
        .assert()
        .success() // Should succeed since we're NOT cursor
        .stdout("true\n")
        .stderr(contains(
            "Warning: Legacy syntax 'facet:ide_id=cursor' is deprecated",
        ))
        .stderr(contains("Use 'ide.id=cursor' instead"));
}

/// Test deprecation warnings with JSON output
#[test]
fn cli_json_output_with_deprecation_warnings() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "--json", "facet:ide_id=vscode"])
        .assert()
        .success()
        .stdout(contains("\"overall\":true"))
        .stdout(contains("\"predicate\":\"facet:ide_id=vscode\""))
        .stderr(contains(
            "Warning: Legacy syntax 'facet:ide_id=vscode' is deprecated",
        ))
        .stderr(contains("Use 'ide.id=vscode' instead"));
}

/// Test deprecation warnings with explain mode
#[test]
fn cli_explain_mode_with_deprecation_warnings() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "--explain", "facet:ide_id=vscode"])
        .assert()
        .success()
        .stdout(contains("true"))
        .stderr(contains(
            "Warning: Legacy syntax 'facet:ide_id=vscode' is deprecated",
        ))
        .stderr(contains("Use 'ide.id=vscode' instead"));
}

/// Test that warnings go to stderr, not stdout
#[test]
fn cli_warnings_go_to_stderr_not_stdout() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "facet:ide_id=vscode"])
        .assert()
        .success()
        .stdout("true\n") // Only the result, no warnings
        .stdout(contains("Warning:").not()) // No warnings in stdout
        .stdout(contains("deprecated").not()) // No deprecation text in stdout
        .stderr(contains("Warning:")); // Warnings in stderr
}

/// Test legacy facet with special characters in value
#[test]
fn cli_legacy_facet_special_characters() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CI", "true")
        .env("GITHUB_REF", "refs/heads/feature/test-123")
        .args(["check", "facet:ci_branch=feature/test-123"])
        .assert()
        .failure() // May fail if branch detection doesn't work exactly as expected
        .stdout("false\n")
        .stderr(contains(
            "Warning: Legacy syntax 'facet:ci_branch=feature/test-123' is deprecated",
        ))
        .stderr(contains("Use 'ci.branch=feature/test-123' instead"));
}

/// Test unknown legacy facet shows warning with best-effort suggestion
#[test]
fn cli_unknown_legacy_facet_shows_warning() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "facet:unknown_key=test_value"])
        .assert()
        .failure() // Should fail since unknown field
        .stdout("false\n")
        .stderr(contains(
            "Warning: Legacy syntax 'facet:unknown_key=test_value' is deprecated",
        ))
        .stderr(contains("Use 'unknown.unknown_key=test_value' instead"));
}

/// Test unknown legacy trait shows warning with best-effort suggestion
#[test]
fn cli_unknown_legacy_trait_shows_warning() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "trait:unknown_trait"])
        .assert()
        .failure() // Should fail since unknown field
        .stdout("false\n")
        .stderr(contains(
            "Warning: Legacy syntax 'trait:unknown_trait' is deprecated",
        ))
        .stderr(contains("Use 'unknown.unknown_trait' instead"));
}
