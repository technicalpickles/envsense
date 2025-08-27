use envsense::detectors::DeclarativeAgentDetector;
use envsense::detectors::{Detection, Detector, EnvSnapshot};
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
    assert_eq!(detection.confidence, 0.9);

    // Check that both agent_id and host facets were set
    assert!(detection.facets_patch.contains_key("agent_id"));
    assert!(detection.facets_patch.contains_key("host"));

    // Verify the values
    let agent_id = detection.facets_patch.get("agent_id").unwrap();
    assert_eq!(agent_id.as_str().unwrap(), "replit-agent");

    let host = detection.facets_patch.get("host").unwrap();
    assert_eq!(host.as_str().unwrap(), "replit");
}
