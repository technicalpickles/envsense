use envsense::detectors::confidence::{HIGH, TERMINAL};
use envsense::schema::{Evidence, Signal};

#[test]
fn test_agent_detection_helper() {
    let evidence = Evidence::agent_detection("CURSOR_AGENT", "1");

    assert_eq!(evidence.signal, Signal::Env);
    assert_eq!(evidence.key, "CURSOR_AGENT");
    assert_eq!(evidence.value, Some("1".to_string()));
    assert_eq!(evidence.supports, vec!["agent.id"]);
    assert_eq!(evidence.confidence, HIGH);
}

#[test]
fn test_agent_with_host_detection_helper() {
    let evidence = Evidence::agent_with_host_detection("REPL_ID", "abc123");

    assert_eq!(evidence.signal, Signal::Env);
    assert_eq!(evidence.key, "REPL_ID");
    assert_eq!(evidence.value, Some("abc123".to_string()));
    assert_eq!(evidence.supports, vec!["agent.id", "host"]);
    assert_eq!(evidence.confidence, HIGH);
}

#[test]
fn test_ide_detection_helper() {
    let evidence = Evidence::ide_detection("TERM_PROGRAM", "vscode");

    assert_eq!(evidence.signal, Signal::Env);
    assert_eq!(evidence.key, "TERM_PROGRAM");
    assert_eq!(evidence.value, Some("vscode".to_string()));
    assert_eq!(evidence.supports, vec!["ide.id"]);
    assert_eq!(evidence.confidence, HIGH);
}

#[test]
fn test_ci_detection_helper() {
    let evidence = Evidence::ci_detection("GITHUB_ACTIONS", "true");

    assert_eq!(evidence.signal, Signal::Env);
    assert_eq!(evidence.key, "GITHUB_ACTIONS");
    assert_eq!(evidence.value, Some("true".to_string()));
    assert_eq!(evidence.supports, vec!["ci.id"]);
    assert_eq!(evidence.confidence, HIGH);
}

#[test]
fn test_ci_multi_field_detection_helper() {
    let evidence =
        Evidence::ci_multi_field_detection("GITHUB_ACTIONS", "true", vec!["id", "vendor"]);

    assert_eq!(evidence.signal, Signal::Env);
    assert_eq!(evidence.key, "GITHUB_ACTIONS");
    assert_eq!(evidence.value, Some("true".to_string()));
    assert_eq!(evidence.supports, vec!["ci.id", "ci.vendor"]);
    assert_eq!(evidence.confidence, HIGH);
}

#[test]
fn test_terminal_stream_tty_helper() {
    let evidence = Evidence::terminal_stream_tty("stdin", true);

    assert_eq!(evidence.signal, Signal::Tty);
    assert_eq!(evidence.key, "terminal.stdin.tty");
    assert_eq!(evidence.value, Some("true".to_string()));
    assert_eq!(evidence.supports, vec!["terminal.stdin.tty"]);
    assert_eq!(evidence.confidence, TERMINAL);
}

#[test]
fn test_terminal_stream_tty_helper_false() {
    let evidence = Evidence::terminal_stream_tty("stdout", false);

    assert_eq!(evidence.signal, Signal::Tty);
    assert_eq!(evidence.key, "terminal.stdout.tty");
    assert_eq!(evidence.value, Some("false".to_string()));
    assert_eq!(evidence.supports, vec!["terminal.stdout.tty"]);
    assert_eq!(evidence.confidence, TERMINAL);
}

#[test]
fn test_terminal_interactive_helper() {
    let evidence = Evidence::terminal_interactive(true);

    assert_eq!(evidence.signal, Signal::Tty);
    assert_eq!(evidence.key, "terminal.interactive");
    assert_eq!(evidence.value, Some("true".to_string()));
    assert_eq!(evidence.supports, vec!["terminal.interactive"]);
    assert_eq!(evidence.confidence, TERMINAL);
}

#[test]
fn test_terminal_color_level_helper() {
    let evidence = Evidence::terminal_color_level("truecolor");

    assert_eq!(evidence.signal, Signal::Tty);
    assert_eq!(evidence.key, "terminal.color_level");
    assert_eq!(evidence.value, Some("truecolor".to_string()));
    assert_eq!(evidence.supports, vec!["terminal.color_level"]);
    assert_eq!(evidence.confidence, TERMINAL);
}

#[test]
fn test_terminal_hyperlinks_helper() {
    let evidence = Evidence::terminal_hyperlinks(true);

    assert_eq!(evidence.signal, Signal::Tty);
    assert_eq!(evidence.key, "terminal.supports_hyperlinks");
    assert_eq!(evidence.value, Some("true".to_string()));
    assert_eq!(evidence.supports, vec!["terminal.supports_hyperlinks"]);
    assert_eq!(evidence.confidence, TERMINAL);
}

#[test]
fn test_helper_methods_can_be_chained_with_confidence_override() {
    let evidence = Evidence::agent_detection("CURSOR_AGENT", "1").with_confidence(0.9);

    assert_eq!(evidence.confidence, 0.9);
    assert_eq!(evidence.supports, vec!["agent.id"]);
}

#[test]
fn test_helper_methods_preserve_other_properties() {
    let evidence = Evidence::terminal_stream_tty("stderr", false);

    // Verify all properties are correctly set
    assert_eq!(evidence.signal, Signal::Tty);
    assert_eq!(evidence.key, "terminal.stderr.tty");
    assert_eq!(evidence.value, Some("false".to_string()));
    assert_eq!(evidence.supports, vec!["terminal.stderr.tty"]);
    assert_eq!(evidence.confidence, TERMINAL);
}

#[test]
fn test_ci_multi_field_detection_with_many_fields() {
    let evidence = Evidence::ci_multi_field_detection(
        "GITHUB_ACTIONS",
        "true",
        vec!["id", "vendor", "name", "is_pr"],
    );

    assert_eq!(
        evidence.supports,
        vec!["ci.id", "ci.vendor", "ci.name", "ci.is_pr"]
    );
}

#[test]
fn test_ci_multi_field_detection_with_empty_fields() {
    let evidence = Evidence::ci_multi_field_detection("CI", "true", vec![]);

    assert_eq!(evidence.supports, Vec::<String>::new());
}
