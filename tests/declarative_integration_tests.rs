use envsense::detectors::{
    DeclarativeAgentDetector, DeclarativeCiDetector, DeclarativeIdeDetector, Detector,
};

use envsense::detectors::test_utils::create_env_snapshot;

/// Test that the declarative system correctly handles priority ordering
#[test]
fn test_ide_priority_ordering() {
    let detector = DeclarativeIdeDetector::new();

    // Test Cursor (highest priority) - should be detected even when VS Code conditions are also met
    let snapshot = create_env_snapshot(vec![
        ("TERM_PROGRAM", "vscode"),
        ("TERM_PROGRAM_VERSION", "1.85.0-insider"), // Would match VS Code Insiders
        ("CURSOR_TRACE_ID", "abc123"),              // But Cursor has higher priority
    ]);

    let detection = detector.detect(&snapshot);
    assert_eq!(
        detection.facets_patch.get("ide_id").unwrap(),
        &serde_json::json!("cursor")
    );
}

/// Test that multiple detectors can work together
#[test]
fn test_multi_detector_integration() {
    // Test environment that should trigger multiple detectors
    let snapshot = create_env_snapshot(vec![
        ("CURSOR_AGENT", "1"),      // Should trigger agent detection
        ("TERM_PROGRAM", "vscode"), // Should trigger IDE detection
        ("GITHUB_ACTIONS", "true"), // Should trigger CI detection
    ]);

    // Test agent detection
    let agent_detector = DeclarativeAgentDetector::new();
    let agent_detection = agent_detector.detect(&snapshot);
    assert!(agent_detection.contexts_add.contains(&"agent".to_string()));
    assert_eq!(
        agent_detection.facets_patch.get("agent_id").unwrap(),
        &serde_json::json!("cursor")
    );

    // Test IDE detection
    let ide_detector = DeclarativeIdeDetector::new();
    let ide_detection = ide_detector.detect(&snapshot);
    assert!(ide_detection.contexts_add.contains(&"ide".to_string()));
    assert_eq!(
        ide_detection.facets_patch.get("ide_id").unwrap(),
        &serde_json::json!("vscode")
    );

    // Test CI detection
    let ci_detector = DeclarativeCiDetector::new();
    let ci_detection = ci_detector.detect(&snapshot);
    assert!(ci_detection.contexts_add.contains(&"ci".to_string()));
    assert_eq!(
        ci_detection.facets_patch.get("ci_id").unwrap(),
        &serde_json::json!("github_actions")
    );
}

/// Test that evidence is properly generated for all detectors
#[test]
fn test_evidence_generation() {
    let snapshot = create_env_snapshot(vec![
        ("CURSOR_AGENT", "1"),
        ("TERM_PROGRAM", "vscode"),
        ("GITHUB_ACTIONS", "true"),
    ]);

    // Test agent evidence
    let agent_detector = DeclarativeAgentDetector::new();
    let agent_detection = agent_detector.detect(&snapshot);
    assert!(!agent_detection.evidence.is_empty());
    assert!(
        agent_detection
            .evidence
            .iter()
            .any(|e| e.key == "CURSOR_AGENT")
    );

    // Test IDE evidence
    let ide_detector = DeclarativeIdeDetector::new();
    let ide_detection = ide_detector.detect(&snapshot);
    assert!(!ide_detection.evidence.is_empty());
    assert!(
        ide_detection
            .evidence
            .iter()
            .any(|e| e.key == "TERM_PROGRAM")
    );

    // Test CI evidence (CI detector doesn't generate evidence for compatibility)
    let ci_detector = DeclarativeCiDetector::new();
    let ci_detection = ci_detector.detect(&snapshot);
    assert!(ci_detection.evidence.is_empty()); // CI detector doesn't generate evidence
}

/// Test that confidence levels are properly assigned
#[test]
fn test_confidence_assignment() {
    let snapshot = create_env_snapshot(vec![
        ("CURSOR_AGENT", "1"),
        ("TERM_PROGRAM", "vscode"),
        ("GITHUB_ACTIONS", "true"),
    ]);

    // All declarative detectors should use HIGH confidence for direct env var matches
    let agent_detector = DeclarativeAgentDetector::new();
    let agent_detection = agent_detector.detect(&snapshot);
    assert_eq!(agent_detection.confidence, 1.0);

    let ide_detector = DeclarativeIdeDetector::new();
    let ide_detection = ide_detector.detect(&snapshot);
    assert_eq!(ide_detection.confidence, 1.0);

    let ci_detector = DeclarativeCiDetector::new();
    let ci_detection = ci_detector.detect(&snapshot);
    assert_eq!(ci_detection.confidence, 1.0);
}

/// Test that contexts are properly assigned
#[test]
fn test_context_assignment() {
    let snapshot = create_env_snapshot(vec![
        ("CURSOR_AGENT", "1"),
        ("TERM_PROGRAM", "vscode"),
        ("GITHUB_ACTIONS", "true"),
    ]);

    let agent_detector = DeclarativeAgentDetector::new();
    let agent_detection = agent_detector.detect(&snapshot);
    assert!(agent_detection.contexts_add.contains(&"agent".to_string()));

    let ide_detector = DeclarativeIdeDetector::new();
    let ide_detection = ide_detector.detect(&snapshot);
    assert!(ide_detection.contexts_add.contains(&"ide".to_string()));

    let ci_detector = DeclarativeCiDetector::new();
    let ci_detection = ci_detector.detect(&snapshot);
    assert!(ci_detection.contexts_add.contains(&"ci".to_string()));
}

/// Test that facets are properly assigned
#[test]
fn test_facet_assignment() {
    let snapshot = create_env_snapshot(vec![
        ("CURSOR_AGENT", "1"),
        ("TERM_PROGRAM", "vscode"),
        ("GITHUB_ACTIONS", "true"),
    ]);

    let agent_detector = DeclarativeAgentDetector::new();
    let agent_detection = agent_detector.detect(&snapshot);
    assert_eq!(
        agent_detection.facets_patch.get("agent_id").unwrap(),
        &serde_json::json!("cursor")
    );

    let ide_detector = DeclarativeIdeDetector::new();
    let ide_detection = ide_detector.detect(&snapshot);
    assert_eq!(
        ide_detection.facets_patch.get("ide_id").unwrap(),
        &serde_json::json!("vscode")
    );

    let ci_detector = DeclarativeCiDetector::new();
    let ci_detection = ci_detector.detect(&snapshot);
    assert_eq!(
        ci_detection.facets_patch.get("ci_id").unwrap(),
        &serde_json::json!("github_actions")
    );
}

/// Test that no detection occurs when no mappings match
#[test]
fn test_no_detection_when_no_mappings_match() {
    let snapshot = create_env_snapshot(vec![
        ("UNKNOWN_VAR", "unknown_value"),
        ("ANOTHER_UNKNOWN", "another_value"),
    ]);

    let agent_detector = DeclarativeAgentDetector::new();
    let agent_detection = agent_detector.detect(&snapshot);
    assert!(agent_detection.contexts_add.is_empty()); // No agent context
    assert!(agent_detection.facets_patch.get("host").is_some()); // Should have default host
    assert_eq!(agent_detection.confidence, 0.0);

    let ide_detector = DeclarativeIdeDetector::new();
    let ide_detection = ide_detector.detect(&snapshot);
    assert!(ide_detection.contexts_add.is_empty());
    assert!(ide_detection.facets_patch.is_empty());
    assert_eq!(ide_detection.confidence, 0.0);

    let ci_detector = DeclarativeCiDetector::new();
    let ci_detection = ci_detector.detect(&snapshot);
    assert!(ci_detection.contexts_add.is_empty());
    assert!(ci_detection.facets_patch.is_empty());
    assert_eq!(ci_detection.confidence, 0.0);
}

/// Test that overrides work correctly across all detectors
#[test]
fn test_override_behavior() {
    // Test ENVSENSE_ASSUME_HUMAN override
    let snapshot = create_env_snapshot(vec![
        ("ENVSENSE_ASSUME_HUMAN", "1"),
        ("CURSOR_AGENT", "1"),
        ("TERM_PROGRAM", "vscode"),
        ("GITHUB_ACTIONS", "true"),
    ]);

    let agent_detector = DeclarativeAgentDetector::new();
    let agent_detection = agent_detector.detect(&snapshot);
    // Should not detect as agent due to override
    assert!(!agent_detection.contexts_add.contains(&"agent".to_string()));

    // IDE and CI detection should still work
    let ide_detector = DeclarativeIdeDetector::new();
    let ide_detection = ide_detector.detect(&snapshot);
    assert!(ide_detection.contexts_add.contains(&"ide".to_string()));

    let ci_detector = DeclarativeCiDetector::new();
    let ci_detection = ci_detector.detect(&snapshot);
    assert!(ci_detection.contexts_add.contains(&"ci".to_string()));
}

/// Test that complex detection scenarios work correctly
#[test]
fn test_complex_detection_scenarios() {
    // Test Replit environment (should detect both agent and host)
    let replit_snapshot = create_env_snapshot(vec![("REPL_ID", "abc123"), ("IS_CODE_AGENT", "1")]);

    let agent_detector = DeclarativeAgentDetector::new();
    let agent_detection = agent_detector.detect(&replit_snapshot);
    assert!(agent_detection.contexts_add.contains(&"agent".to_string()));
    assert_eq!(
        agent_detection.facets_patch.get("agent_id").unwrap(),
        &serde_json::json!("replit-agent")
    );
    assert_eq!(
        agent_detection.facets_patch.get("host").unwrap(),
        &serde_json::json!("replit")
    );

    // Test Cursor environment (should detect both agent and IDE)
    let cursor_snapshot = create_env_snapshot(vec![
        ("CURSOR_AGENT", "1"),
        ("TERM_PROGRAM", "vscode"),
        ("CURSOR_TRACE_ID", "abc123"),
    ]);

    let agent_detection = agent_detector.detect(&cursor_snapshot);
    assert!(agent_detection.contexts_add.contains(&"agent".to_string()));
    assert_eq!(
        agent_detection.facets_patch.get("agent_id").unwrap(),
        &serde_json::json!("cursor")
    );

    let ide_detector = DeclarativeIdeDetector::new();
    let ide_detection = ide_detector.detect(&cursor_snapshot);
    assert!(ide_detection.contexts_add.contains(&"ide".to_string()));
    assert_eq!(
        ide_detection.facets_patch.get("ide_id").unwrap(),
        &serde_json::json!("cursor")
    );
}
