use envsense::detectors::{DeclarativeAgentDetector, DeclarativeIdeDetector};
use envsense::detectors::{Detector, EnvSnapshot};
use std::collections::HashMap;

#[test]
fn test_declarative_agent_detector_integration() {
    // Test that the DeclarativeAgentDetector can be used
    let detector = DeclarativeAgentDetector::new();

    // Create a test environment with Cursor agent
    let mut env_vars = HashMap::new();
    env_vars.insert("CURSOR_AGENT".to_string(), "1".to_string());

    let snapshot = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

    // Run detection
    let detection = detector.detect(&snapshot);

    // Verify it detected cursor
    assert!(detection.contexts_add.contains(&"agent".to_string()));
    assert_eq!(detection.confidence, 1.0);

    // Check that agent_id facet was set
    assert!(detection.facets_patch.contains_key("agent_id"));

    // Check that evidence was generated
    assert!(!detection.evidence.is_empty());
}

#[test]
fn test_declarative_agent_detector_with_replit() {
    let detector = DeclarativeAgentDetector::new();

    // Create a test environment with Replit
    let mut env_vars = HashMap::new();
    env_vars.insert("REPL_ID".to_string(), "abc123".to_string());

    let snapshot = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

    // Run detection
    let detection = detector.detect(&snapshot);

    // Verify it detected replit agent
    assert!(detection.contexts_add.contains(&"agent".to_string()));
    assert_eq!(detection.confidence, 1.0);

    // Check that both agent_id and host facets were set
    assert!(detection.facets_patch.contains_key("agent_id"));
    assert!(detection.facets_patch.contains_key("host"));

    // Verify the values
    let agent_id = detection.facets_patch.get("agent_id").unwrap();
    assert_eq!(agent_id.as_str().unwrap(), "replit-agent");

    let host = detection.facets_patch.get("host").unwrap();
    assert_eq!(host.as_str().unwrap(), "replit");
}

#[test]
fn test_declarative_ide_detector_integration() {
    let detector = DeclarativeIdeDetector::new();

    // Create a test environment with VSCode
    let mut env_vars = HashMap::new();
    env_vars.insert("TERM_PROGRAM".to_string(), "vscode".to_string());
    env_vars.insert("TERM_PROGRAM_VERSION".to_string(), "1.85.0".to_string());

    let snapshot = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

    // Run detection
    let detection = detector.detect(&snapshot);

    // Verify it detected IDE context
    assert!(detection.contexts_add.contains(&"ide".to_string()));
    assert_eq!(detection.confidence, 1.0);

    // Check that nested IDE traits object was created
    assert!(detection.traits_patch.contains_key("ide"));
    let ide_value = detection.traits_patch.get("ide").unwrap();
    let ide_traits: envsense::traits::IdeTraits =
        serde_json::from_value(ide_value.clone()).unwrap();
    assert_eq!(ide_traits.id, Some("vscode".to_string()));

    // Check that legacy ide_id facet was set for backward compatibility
    assert!(detection.facets_patch.contains_key("ide_id"));
    let ide_id = detection.facets_patch.get("ide_id").unwrap();
    assert_eq!(ide_id.as_str().unwrap(), "vscode");

    // Check that evidence was generated with nested field paths
    assert!(!detection.evidence.is_empty());
    let ide_evidence = detection
        .evidence
        .iter()
        .find(|e| e.key == "TERM_PROGRAM")
        .expect("Should find TERM_PROGRAM evidence");
    assert!(ide_evidence.supports.contains(&"ide".to_string()));
    assert!(ide_evidence.supports.contains(&"ide.id".to_string()));
}

#[test]
fn test_declarative_ide_detector_with_cursor() {
    let detector = DeclarativeIdeDetector::new();

    // Create a test environment with Cursor
    let mut env_vars = HashMap::new();
    env_vars.insert("TERM_PROGRAM".to_string(), "vscode".to_string());
    env_vars.insert("CURSOR_TRACE_ID".to_string(), "abc123".to_string());

    let snapshot = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

    // Run detection
    let detection = detector.detect(&snapshot);

    // Verify it detected Cursor as IDE
    assert!(detection.contexts_add.contains(&"ide".to_string()));
    assert_eq!(detection.confidence, 1.0);

    // Check nested IDE traits object
    assert!(detection.traits_patch.contains_key("ide"));
    let ide_value = detection.traits_patch.get("ide").unwrap();
    let ide_traits: envsense::traits::IdeTraits =
        serde_json::from_value(ide_value.clone()).unwrap();
    assert_eq!(ide_traits.id, Some("cursor".to_string()));

    // Check legacy facet
    let ide_id = detection.facets_patch.get("ide_id").unwrap();
    assert_eq!(ide_id.as_str().unwrap(), "cursor");

    // Verify evidence for both TERM_PROGRAM and CURSOR_TRACE_ID
    assert_eq!(detection.evidence.len(), 2);
    for evidence in &detection.evidence {
        assert!(evidence.supports.contains(&"ide".to_string()));
        assert!(evidence.supports.contains(&"ide.id".to_string()));
    }
}
