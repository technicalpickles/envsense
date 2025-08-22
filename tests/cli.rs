use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin;
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
    let bin = cargo_bin("envsense");

    let mut cmd = Command::new("script");
    cmd.arg("-qec")
        .arg(format!("{} info --fields=traits", bin.display()))
        .arg("/dev/null")
        .env("TERM", "xterm");
    cmd.assert()
        .success()
        .stdout(contains("\u{1b}[1;36m").and(contains("\u{1b}[31m")));

    let mut cmd = Command::new("script");
    cmd.arg("-qec")
        .arg(format!("{} info --fields=traits", bin.display()))
        .arg("/dev/null")
        .env("TERM", "xterm")
        .env("NO_COLOR", "1");
    cmd.assert()
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
        .stdout(contains("\"schema_version\"").and(contains("\"rules_version\"")));

    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["info", "--fields=meta", "--no-color"])
        .assert()
        .success()
        .stdout(contains(
            "Meta:\n  rules_version = \n  schema_version = 0.1.0",
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
