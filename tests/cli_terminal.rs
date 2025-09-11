use assert_cmd::Command;

#[test]
fn info_reports_piped_stdout() {
    let output = Command::cargo_bin("envsense")
        .unwrap()
        .arg("info")
        .output()
        .expect("failed to run envsense");
    let text = String::from_utf8_lossy(&output.stdout);
    // Check for the new hierarchical format
    assert!(
        text.contains("stdout:") && text.contains("piped: true") && text.contains("tty: false")
    );
}
