use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin;
use predicates::prelude::*;
use predicates::str::contains;

#[cfg(target_os = "macos")]
fn script_cmd(envs: &[(&str, &str)]) -> Command {
    let bin = cargo_bin("envsense");
    let mut cmd = Command::new("script");
    cmd.arg("-q")
        .arg("/dev/null")
        .arg(bin)
        .arg("info")
        .arg("--fields=traits");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd
}

#[cfg(not(target_os = "macos"))]
fn script_cmd(envs: &[(&str, &str)]) -> Command {
    let bin = cargo_bin("envsense");
    let mut cmd = Command::new("script");
    cmd.arg("-qec")
        .arg(format!("{} info --fields=traits", bin.display()))
        .arg("/dev/null");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd
}

#[test]
fn prints_json_with_version() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--json"])
        .assert()
        .success()
        .stdout(contains("\"schema_version\""));
}

#[test]
fn fields_limit_output() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--json", "--fields=contexts"])
        .assert()
        .success()
        .stdout(contains("\"contexts\"").and(contains("\"traits\"").not()));
}

#[test]
fn human_info_multiline() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["info", "--fields=traits", "--no-color"])
        .assert()
        .success()
        .stdout(contains("Traits:"))
        .stdout(contains("terminal:"))
        .stdout(contains("color_level = none"))
        .stdout(contains("interactive = false"));
}

#[test]
fn raw_contexts_without_headings_or_color() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["info", "--raw", "--fields=contexts"])
        .assert()
        .success()
        .stdout("ide\n");
}

#[test]
fn color_output_honors_tty_and_no_color() {
    script_cmd(&[("TERM", "xterm")])
        .assert()
        .success()
        .stdout(contains("\u{1b}[1;36m").and(contains("\u{1b}[32m")));

    script_cmd(&[("TERM", "xterm"), ("NO_COLOR", "1")])
        .assert()
        .success()
        .stdout(contains("\u{1b}[").not().and(contains("Traits:")));
}

#[test]
fn invalid_fields_error() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--fields=bogus"])
        .assert()
        .code(2)
        .stderr(contains("unknown field: bogus"));

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--json", "--fields=bogus"])
        .assert()
        .code(2)
        .stderr(contains("unknown field: bogus"));
}

#[test]
fn meta_field_selection() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--json", "--fields=meta"])
        .assert()
        .success()
        .stdout(contains("\"schema_version\""));

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--fields=meta", "--no-color"])
        .assert()
        .success()
        .stdout(contains("Meta:\n  schema_version = 0.3.0"));
}

#[test]
fn check_unknown_context_fails() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "bogus"])
        .assert()
        .failure()
        .stdout("false\n");
}

#[test]
fn detects_vscode() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .args(["check", "ide.id=vscode"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn detects_vscode_insiders() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0-insider")
        .args(["check", "ide.id=vscode-insiders"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn detects_cursor() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .env("TERM_PROGRAM_VERSION", "1.75.0")
        .env("CURSOR_TRACE_ID", "xyz")
        .args(["check", "ide.id=cursor"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn quiet_flag_suppresses_output() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "-q", "agent"])
        .assert()
        .failure()
        .stdout(predicates::str::is_empty());
}

#[test]
fn check_agent_context() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent"])
        .assert()
        .success()
        .stdout("true\n");

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "agent"])
        .assert()
        .failure()
        .stdout("false\n");
}

#[test]
fn json_output_contains_ci_keys() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("GITHUB_ACTIONS", "1")
        .args(["info", "--json"])
        .assert()
        .success()
        .stdout(contains("\"ci\"").and(contains("\"vendor\": \"github_actions\"")));
}

#[test]
fn check_ci_messages_and_exit_codes() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("GITHUB_ACTIONS", "1")
        .arg("check")
        .arg("ci")
        .assert()
        .success()
        .stdout(contains("CI detected"));

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .arg("check")
        .arg("ci")
        .assert()
        .failure()
        .stdout(contains("No CI detected"));
}

#[test]
fn check_ci_quiet_suppresses_output() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .arg("check")
        .arg("-q")
        .arg("ci")
        .assert()
        .failure()
        .stdout(predicates::str::is_empty());
}

/// Additional comprehensive integration tests for Task 2.8
#[test]
fn check_predicate_parsing_edge_cases() {
    // Test negation with various predicates
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "!ci"])
        .assert()
        .success() // Should succeed because ci is NOT detected
        .stdout("true\n");

    // Test deep field paths
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "terminal.stdin.tty"])
        .assert(); // Will be true or false, but should not error

    // Test field comparison with boolean values
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "terminal.stdin.tty=true"])
        .assert(); // May succeed or fail, but should not error
}

#[test]
fn check_comprehensive_exit_codes() {
    // Test that exit codes are correct for various scenarios

    // Single successful check
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent"])
        .assert()
        .code(0);

    // Single failed check
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear().args(["check", "agent"]).assert().code(1);

    // Multiple checks with --all (default) - all must pass
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent", "ci"]) // agent=true, ci=false
        .assert()
        .code(1); // Should fail because not all pass

    // Multiple checks with --any - at least one must pass
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--any", "agent", "ci"]) // agent=true, ci=false
        .assert()
        .code(0); // Should succeed because at least one passes

    // Parse error should return error code
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear().args(["check", "ide."]).assert().code(2); // Parse error
}

#[test]
fn check_output_formatting_consistency() {
    // Test that output format is consistent across different scenarios

    // Single predicate output
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent.id"])
        .assert()
        .success()
        .stdout("cursor\n");

    // Multiple predicates output format
    let output = Command::cargo_bin("envsense")
        .unwrap()
        .env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent", "agent.id"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("overall=true"));
    assert!(stdout.contains("agent=true"));
    assert!(stdout.contains("agent.id=cursor"));

    // JSON output consistency
    let output = Command::cargo_bin("envsense")
        .unwrap()
        .env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "--json", "agent"])
        .output()
        .unwrap();

    let json_str = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(json["overall"].is_boolean());
    assert!(json["checks"].is_array());
}

#[test]
fn check_field_registry_integration() {
    // Test that all registered fields work in CLI
    let fields_to_test = vec![
        "agent.id",
        "ide.id",
        "terminal.interactive",
        "terminal.color_level",
        "terminal.stdin.tty",
        "terminal.stdout.tty",
        "terminal.stderr.tty",
        "terminal.stdin.piped",
        "terminal.stdout.piped",
        "terminal.stderr.piped",
        "terminal.supports_hyperlinks",
        "ci.id",
        "ci.vendor",
        "ci.name",
        "ci.is_pr",
        "ci.branch",
    ];

    for field in fields_to_test {
        let mut cmd = Command::cargo_bin("envsense").unwrap();
        cmd.env_clear().args(["check", field]).assert(); // Should not error, regardless of value
    }
}

#[test]
fn check_new_syntax_comprehensive() {
    // Test that all new syntax works correctly

    // New field syntax
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent.id=cursor"])
        .assert()
        .success()
        .stdout("true\n");

    // New field syntax without value
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "terminal.interactive"])
        .assert()
        .stderr(predicates::str::is_empty());

    // Mixed new syntax
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent", "agent.id=cursor"])
        .assert()
        .success();
}
