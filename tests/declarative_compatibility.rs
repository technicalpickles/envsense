use envsense::detectors::DeclarativeAgentDetector;
use envsense::detectors::{Detector, EnvSnapshot};
use std::collections::HashMap;

/// Test that the declarative agent detector produces the same results as the old system
/// This uses the same test cases as the original table_driven_detection test
#[test]
fn declarative_compatibility_with_original() {
    struct Case {
        name: &'static str,
        env: Vec<(&'static str, &'static str)>,
        expected_agent: Option<&'static str>,
        expected_is_agent: bool,
    }

    let cases = vec![
        Case {
            name: "cursor_terminal",
            env: vec![("CURSOR_AGENT", "1"), ("TERM_PROGRAM", "vscode")],
            expected_agent: Some("cursor"),
            expected_is_agent: true,
        },
        Case {
            name: "cline_basic",
            env: vec![("CLINE_ACTIVE", "true")],
            expected_agent: Some("cline"),
            expected_is_agent: true,
        },
        Case {
            name: "claude_code",
            env: vec![("CLAUDECODE", "1")],
            expected_agent: Some("claude-code"),
            expected_is_agent: true,
        },
        Case {
            name: "replit_full",
            env: vec![("REPL_ID", "abc"), ("REPLIT_USER", "josh")],
            expected_agent: Some("replit-agent"),
            expected_is_agent: true,
        },
        Case {
            name: "replit_weak",
            env: vec![("REPLIT_USER", "josh")],
            expected_agent: None,
            expected_is_agent: false,
        },
        Case {
            name: "openhands",
            env: vec![
                ("SANDBOX_VOLUMES", "..."),
                ("SANDBOX_RUNTIME_CONTAINER_IMAGE", "..."),
            ],
            expected_agent: Some("openhands"),
            expected_is_agent: true,
        },
        Case {
            name: "aider",
            env: vec![("AIDER_MODEL", "gpt-4o-mini")],
            expected_agent: Some("aider"),
            expected_is_agent: true,
        },
        Case {
            name: "vscode_only",
            env: vec![("TERM_PROGRAM", "vscode")],
            expected_agent: None,
            expected_is_agent: false,
        },
        Case {
            name: "override_force_human",
            env: vec![("ENVSENSE_ASSUME_HUMAN", "1"), ("CURSOR_AGENT", "1")],
            expected_agent: None,
            expected_is_agent: false,
            // When assume human, no host is set
        },
        Case {
            name: "override_force_agent",
            env: vec![("ENVSENSE_AGENT", "cursor")],
            expected_agent: Some("cursor"),
            expected_is_agent: true,
        },
    ];

    let detector = DeclarativeAgentDetector::new();

    for case in cases {
        // Create environment snapshot
        let mut env_vars = HashMap::new();
        for (k, v) in &case.env {
            env_vars.insert(k.to_string(), v.to_string());
        }
        let snapshot = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

        // Run detection
        let detection = detector.detect(&snapshot);

        // Check agent detection
        let actual_is_agent = detection.contexts_add.contains(&"agent".to_string());
        assert_eq!(
            actual_is_agent, case.expected_is_agent,
            "{}: expected is_agent={}, got {}",
            case.name, case.expected_is_agent, actual_is_agent
        );

        // Check agent ID
        let actual_agent_id = detection
            .facets_patch
            .get("agent_id")
            .and_then(|v| v.as_str());
        assert_eq!(
            actual_agent_id, case.expected_agent,
            "{}: expected agent_id={:?}, got {:?}",
            case.name, case.expected_agent, actual_agent_id
        );

        // Host concept removed - no longer checking host

        // Verify evidence is generated when agent is detected
        if case.expected_is_agent {
            assert!(
                !detection.evidence.is_empty(),
                "{}: expected evidence for agent detection",
                case.name
            );
        }
    }
}

/// Test that the declarative system handles edge cases correctly
#[test]
fn declarative_edge_cases() {
    let detector = DeclarativeAgentDetector::new();

    // Test empty environment
    let snapshot = EnvSnapshot::with_mock_tty(HashMap::new(), false, false, false);
    let detection = detector.detect(&snapshot);
    assert!(!detection.contexts_add.contains(&"agent".to_string()));
    // Host concept removed - no longer expecting host facet

    // Test multiple agent indicators (currently picks first match, not highest confidence)
    let mut env_vars = HashMap::new();
    env_vars.insert("CURSOR_AGENT".to_string(), "1".to_string());
    env_vars.insert("REPL_ID".to_string(), "abc123".to_string());
    let snapshot = EnvSnapshot::with_mock_tty(env_vars, false, false, false);
    let detection = detector.detect(&snapshot);
    // Currently picks replit (first in list) over cursor (higher confidence)
    // This is a known limitation - should be fixed to pick highest confidence
    assert_eq!(
        detection
            .facets_patch
            .get("agent_id")
            .unwrap()
            .as_str()
            .unwrap(),
        "replit-agent"
    );

    // Test override with "none"
    let mut env_vars = HashMap::new();
    env_vars.insert("ENVSENSE_AGENT".to_string(), "none".to_string());
    env_vars.insert("CURSOR_AGENT".to_string(), "1".to_string());
    let snapshot = EnvSnapshot::with_mock_tty(env_vars, false, false, false);
    let detection = detector.detect(&snapshot);
    assert!(!detection.contexts_add.contains(&"agent".to_string()));
}

/// Test that evidence generation works correctly
#[test]
fn declarative_evidence_generation() {
    let detector = DeclarativeAgentDetector::new();

    // Test cursor detection evidence
    let mut env_vars = HashMap::new();
    env_vars.insert("CURSOR_AGENT".to_string(), "1".to_string());
    let snapshot = EnvSnapshot::with_mock_tty(env_vars, false, false, false);
    let detection = detector.detect(&snapshot);

    assert!(!detection.evidence.is_empty());
    let cursor_evidence = detection
        .evidence
        .iter()
        .find(|e| e.key == "CURSOR_AGENT")
        .expect("Should have CURSOR_AGENT evidence");
    assert_eq!(cursor_evidence.value.as_deref(), Some("1"));
    // Note: Agent detector currently only includes nested field path in supports
    assert!(cursor_evidence.supports.contains(&"agent.id".to_string()));
    assert_eq!(cursor_evidence.confidence, 1.0);

    // Test replit detection evidence
    let mut env_vars = HashMap::new();
    env_vars.insert("REPL_ID".to_string(), "abc123".to_string());
    let snapshot = EnvSnapshot::with_mock_tty(env_vars, false, false, false);
    let detection = detector.detect(&snapshot);

    assert!(!detection.evidence.is_empty());
    let replit_evidence = detection
        .evidence
        .iter()
        .find(|e| e.key == "REPL_ID")
        .expect("Should have REPL_ID evidence");
    assert_eq!(replit_evidence.value.as_deref(), Some("abc123"));
    // Note: Agent detector currently only includes nested field path in supports
    assert!(replit_evidence.supports.contains(&"agent.id".to_string()));
    assert_eq!(replit_evidence.confidence, 1.0);
}
