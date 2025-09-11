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

/// Additional snapshot tests for Task 2.8
#[test]
fn snapshot_help_text_stability() {
    // Test that help text remains stable across changes
    let output = Command::cargo_bin("envsense")
        .unwrap()
        .args(["check", "--help"])
        .output()
        .expect("failed to run envsense check --help");

    assert!(output.status.success());
    let help_text = String::from_utf8(output.stdout).unwrap();

    // Verify key sections are present
    assert!(help_text.contains("Available predicates:"));
    assert!(help_text.contains("Contexts (return boolean):"));
    assert!(help_text.contains("Fields:"));
    assert!(help_text.contains("Examples:"));
    assert!(help_text.contains("Syntax:"));

    // Verify all contexts are documented
    assert!(help_text.contains("agent                    # Check if agent context is detected"));
    assert!(help_text.contains("ide                    # Check if ide context is detected"));
    assert!(
        help_text.contains("terminal                    # Check if terminal context is detected")
    );
    assert!(help_text.contains("ci                    # Check if ci context is detected"));

    // Verify field categories are present
    assert!(help_text.contains("agent fields:"));
    assert!(help_text.contains("ide fields:"));
    assert!(help_text.contains("terminal fields:"));
    assert!(help_text.contains("ci fields:"));

    // Verify examples cover different usage patterns
    assert!(help_text.contains("envsense check agent              # Boolean: is agent detected?"));
    assert!(help_text.contains("envsense check agent.id           # String: show agent ID"));
    assert!(
        help_text.contains("envsense check agent.id=cursor    # Boolean: is agent ID 'cursor'?")
    );
    assert!(help_text.contains("envsense check !ci                # Boolean: is CI NOT detected?"));
}

#[test]
fn snapshot_error_messages() {
    // Test that error messages are consistent and helpful

    // Invalid field path error
    let output = Command::cargo_bin("envsense")
        .unwrap()
        .env_clear()
        .args(["check", "invalid.field"])
        .output()
        .expect("failed to run envsense");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error parsing"));

    // Malformed field syntax error
    let output = Command::cargo_bin("envsense")
        .unwrap()
        .env_clear()
        .args(["check", "agent."])
        .output()
        .expect("failed to run envsense");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error parsing"));

    // Empty predicate error
    let output = Command::cargo_bin("envsense")
        .unwrap()
        .env_clear()
        .args(["check", ""])
        .output()
        .expect("failed to run envsense");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error parsing"));
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

#[test]
fn snapshot_os_linux() {
    let json = run_info_json(&[("OSTYPE", "linux")]);
    assert_json_snapshot!("os_linux", json);
}

#[test]
fn snapshot_nested_structure_comprehensive() {
    // Test comprehensive nested structure with multiple detectors active
    let json = run_info_json(&[
        ("CURSOR_AGENT", "1"),
        ("TERM_PROGRAM", "vscode"),
        ("GITHUB_ACTIONS", "true"),
        ("GITHUB_REF", "refs/heads/main"),
    ]);

    // Verify that the JSON has the expected structure
    assert!(json.get("traits").is_some(), "Missing 'traits' in JSON");
    assert!(json.get("contexts").is_some(), "Missing 'contexts' in JSON");
    assert!(json.get("evidence").is_some(), "Missing 'evidence' in JSON");

    let traits = json["traits"].as_object().expect("traits should be object");

    // With the new nested structure, we expect context-based organization
    assert!(
        traits.contains_key("terminal"),
        "Missing 'terminal' context in traits"
    );
    assert!(
        traits.contains_key("agent"),
        "Missing 'agent' context in traits"
    );
    assert!(
        traits.contains_key("ide"),
        "Missing 'ide' context in traits"
    );

    // Verify terminal traits are nested under terminal context
    let terminal = traits["terminal"]
        .as_object()
        .expect("terminal should be object");
    assert!(
        terminal.contains_key("interactive"),
        "Missing 'interactive' in terminal traits"
    );
    assert!(
        terminal.contains_key("stdin"),
        "Missing 'stdin' in terminal traits"
    );

    // Verify evidence uses nested field paths (this is the key Phase 3 change)
    let evidence_array = json["evidence"]
        .as_array()
        .expect("evidence should be array");
    let has_nested_evidence = evidence_array.iter().any(|e| {
        e.get("supports")
            .and_then(|s| s.as_array())
            .map_or(false, |supports| {
                supports
                    .iter()
                    .any(|support| support.as_str().map_or(false, |s| s.contains(".")))
            })
    });
    assert!(
        has_nested_evidence,
        "Evidence should use nested field paths like 'terminal.stdin.tty'"
    );

    // Use existing insta snapshot testing for nested structure regression
    assert_json_snapshot!("nested_structure_comprehensive", json);
}

#[test]
fn snapshot_os_macos() {
    let json = run_info_json(&[("OSTYPE", "darwin")]);
    assert_json_snapshot!("os_macos", json);
}

#[test]
fn snapshot_check_list_output() {
    let mut cmd = Command::cargo_bin("envsense").unwrap();
    cmd.env_clear();
    cmd.args(["check", "--list"]);

    let output = cmd.output().expect("failed to run envsense check --list");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    insta::assert_snapshot!("check_list_output", stdout);
}
