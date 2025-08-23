use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

#[test]
fn info_json_has_keys() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--json"]).assert().success().stdout(
        contains("\"contexts\"")
            .and(contains("\"traits\""))
            .and(contains("\"facets\""))
            .and(contains("\"meta\""))
            .and(contains("\"evidence\"")),
    );
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
        .env("VSCODE_PID", "1")
        .args(["check", "facet:ide_id=vscode"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn quiet_flag_suppresses_output() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "-q", "agent"])
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
