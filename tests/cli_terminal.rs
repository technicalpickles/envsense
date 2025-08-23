use assert_cmd::Command;

#[test]
fn info_reports_piped_stdout() {
    let output = Command::cargo_bin("envsense")
        .unwrap()
        .arg("info")
        .output()
        .expect("failed to run envsense");
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("is_tty_stdout = false"));
    assert!(text.contains("is_piped_stdout = true"));
}
