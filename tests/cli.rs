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
        .stdout(contains(
            "Traits:\n  color_level = none\n  is_ci = false\n  is_interactive = false",
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
    script_cmd(&[("TERM", "xterm")])
        .assert()
        .success()
        .stdout(contains("\u{1b}[1;36m").and(contains("\u{1b}[31m")));

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
        .stdout(contains("\"ci\"").and(contains("ci_vendor")));
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
