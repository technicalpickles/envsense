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
