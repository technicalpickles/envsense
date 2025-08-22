use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn prints_hello_world() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.assert().success().stdout(contains("Hello, world!"));
}
