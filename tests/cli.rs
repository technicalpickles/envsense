use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn prints_json_with_version() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.arg("--json")
        .assert()
        .success()
        .stdout(contains("\"version\":\"0.1.0\""));
}

#[test]
fn check_unknown_context_fails() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.args(["--check", "agent"])
        .assert()
        .failure();
}
