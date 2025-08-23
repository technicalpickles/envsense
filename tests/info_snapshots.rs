use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin;
use insta::assert_json_snapshot;
use serde_json::Value;

fn parse_json(bytes: &[u8]) -> Value {
    let start = bytes
        .iter()
        .position(|&b| b == b'{')
        .expect("json start not found");
    serde_json::from_slice(&bytes[start..]).expect("invalid json")
}

fn run_info_json(envs: &[(&str, &str)]) -> Value {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear();
    cmd.args(["info", "--json"]);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let output = cmd.output().expect("failed to run envsense");
    assert!(output.status.success());
    parse_json(&output.stdout)
}

#[cfg(target_os = "macos")]
fn run_info_json_tty(envs: &[(&str, &str)]) -> Value {
    let bin = cargo_bin("envsense");
    let mut cmd = Command::new("script");
    cmd.arg("-q")
        .arg("/dev/null")
        .arg(bin)
        .arg("info")
        .arg("--json");
    cmd.env_clear();
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let output = cmd.output().expect("failed to run script");
    assert!(output.status.success());
    parse_json(&output.stdout)
}

#[cfg(not(target_os = "macos"))]
fn run_info_json_tty(envs: &[(&str, &str)]) -> Value {
    let bin = cargo_bin("envsense");
    let mut cmd = Command::new("script");
    cmd.arg("-qec")
        .arg(format!("{} info --json", bin.display()))
        .arg("/dev/null");
    cmd.env_clear();
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let output = cmd.output().expect("failed to run script");
    assert!(output.status.success());
    parse_json(&output.stdout)
}

#[test]
fn snapshot_vscode() {
    let json = run_info_json(&[
        ("TERM_PROGRAM", "vscode"),
        ("TERM_PROGRAM_VERSION", "1.75.0"),
    ]);
    assert_json_snapshot!("vscode", json);
}

#[test]
fn snapshot_vscode_insiders() {
    let json = run_info_json(&[
        ("TERM_PROGRAM", "vscode"),
        ("TERM_PROGRAM_VERSION", "1.75.0-insider"),
    ]);
    assert_json_snapshot!("vscode_insiders", json);
}

#[test]
fn snapshot_cursor() {
    let json = run_info_json(&[
        ("TERM_PROGRAM", "vscode"),
        ("TERM_PROGRAM_VERSION", "1.75.0"),
        ("CURSOR_TRACE_ID", "xyz"),
    ]);
    assert_json_snapshot!("cursor", json);
}

#[test]
fn snapshot_github_actions() {
    let json = run_info_json(&[("GITHUB_ACTIONS", "1")]);
    assert_json_snapshot!("github_actions", json);
}

#[test]
fn snapshot_gitlab_ci() {
    let json = run_info_json(&[("GITLAB_CI", "1")]);
    assert_json_snapshot!("gitlab_ci", json);
}

#[test]
fn snapshot_tmux() {
    let json = run_info_json_tty(&[("TERM", "screen-256color"), ("TMUX", "1")]);
    assert_json_snapshot!("tmux", json);
}

#[test]
fn snapshot_plain_tty() {
    let json = run_info_json_tty(&[("TERM", "xterm-256color")]);
    assert_json_snapshot!("plain_tty", json);
}

#[test]
fn snapshot_piped_io() {
    let json = run_info_json(&[]);
    assert_json_snapshot!("piped_io", json);
}

#[test]
fn snapshot_shell_bash() {
    let json = run_info_json(&[("SHELL", "/bin/bash")]);
    assert_json_snapshot!("shell_bash", json);
}

#[test]
fn snapshot_shell_zsh() {
    let json = run_info_json(&[("SHELL", "/bin/zsh")]);
    assert_json_snapshot!("shell_zsh", json);
}
