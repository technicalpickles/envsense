use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

/// Test that the declarative system works with CLI commands
/// These tests mirror the existing CLI tests but focus on agent detection
#[test]
fn cli_declarative_cursor_detection() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent"])
        .assert()
        .success()
        .stdout("true\n");

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["check", "facet:agent_id=cursor"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn cli_declarative_replit_detection() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("REPL_ID", "abc123")
        .args(["check", "agent"])
        .assert()
        .success()
        .stdout("true\n");

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("REPL_ID", "abc123")
        .args(["check", "facet:agent_id=replit-agent"])
        .assert()
        .success()
        .stdout("true\n");

    // Test host detection
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("REPLIT_USER", "josh")
        .args(["info", "--json"])
        .assert()
        .success()
        .stdout(contains("\"host\"").and(contains("replit")));
}

#[test]
fn cli_declarative_claude_code_detection() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CLAUDECODE", "1")
        .args(["check", "agent"])
        .assert()
        .success()
        .stdout("true\n");

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CLAUDECODE", "1")
        .args(["check", "facet:agent_id=claude-code"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn cli_declarative_aider_detection() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("AIDER_MODEL", "gpt-4o-mini")
        .args(["check", "agent"])
        .assert()
        .success()
        .stdout("true\n");

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("AIDER_MODEL", "gpt-4o-mini")
        .args(["check", "facet:agent_id=aider"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn cli_declarative_openhands_detection() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("SANDBOX_VOLUMES", "/tmp")
        .env("SANDBOX_RUNTIME_CONTAINER_IMAGE", "alpine")
        .args(["check", "agent"])
        .assert()
        .success()
        .stdout("true\n");

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("SANDBOX_VOLUMES", "/tmp")
        .args(["check", "facet:agent_id=openhands"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn cli_declarative_overrides() {
    // Test ENVSENSE_ASSUME_HUMAN override
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("ENVSENSE_ASSUME_HUMAN", "1")
        .env("CURSOR_AGENT", "1")
        .args(["check", "agent"])
        .assert()
        .failure()
        .stdout("false\n");

    // Test ENVSENSE_AGENT override
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("ENVSENSE_AGENT", "custom-agent")
        .args(["check", "agent"])
        .assert()
        .success()
        .stdout("true\n");

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("ENVSENSE_AGENT", "custom-agent")
        .args(["check", "facet:agent_id=custom-agent"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn cli_declarative_no_agent_detection() {
    // Test that no agent is detected in clean environment
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .args(["check", "agent"])
        .assert()
        .failure()
        .stdout("false\n");

    // Test that only IDE detection works without agent
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("TERM_PROGRAM", "vscode")
        .args(["check", "ide"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn cli_declarative_evidence_in_json() {
    // Test that evidence is included in JSON output
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("CURSOR_AGENT", "1")
        .args(["info", "--json"])
        .assert()
        .success()
        .stdout(contains("\"evidence\"").and(contains("CURSOR_AGENT")));

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear()
        .env("REPL_ID", "abc123")
        .args(["info", "--json"])
        .assert()
        .success()
        .stdout(contains("\"evidence\"").and(contains("REPL_ID")));
}
