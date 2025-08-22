use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

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
        .stdout(contains(
            "Traits:\n  color_level = none\n  is_interactive = false",
        ));
}

#[test]
fn check_unknown_context_fails() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["check", "agent"])
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
        .args(["check", "facet:ide_id=vscode"])
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
        .args(["check", "facet:ide_id=vscode-insiders"])
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
        .args(["check", "facet:ide_id=cursor"])
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
