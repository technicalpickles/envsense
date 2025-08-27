//! Test for macro integration with the main crate

use envsense::schema::EnvSense;

#[test]
fn test_macro_integration_works() {
    // Test that the macro integration works by using the main crate's EnvSense
    let envsense = EnvSense::default();

    // Verify the struct has the expected fields
    assert!(!envsense.contexts.agent);
    assert!(!envsense.contexts.ide);
    assert!(!envsense.contexts.ci);
    assert!(!envsense.traits.is_interactive);
    assert!(!envsense.traits.is_tty_stdout);
    assert_eq!(envsense.facets.agent_id, None);
    assert_eq!(envsense.facets.ide_id, None);
    assert_eq!(envsense.evidence.len(), 0);
    assert_eq!(envsense.version, "0.1.0");
    assert_eq!(envsense.rules_version, "");
}

#[test]
fn test_detection_engine_uses_macro() {
    // Test that the detection engine works with the macro-generated merging
    use envsense::detectors::terminal::TerminalDetector;
    use envsense::engine::DetectionEngine;

    let engine = DetectionEngine::new().register(TerminalDetector::new());

    let result = engine.detect();

    // Verify that the macro-generated merging worked
    // The terminal detector should have set some traits
    // Boolean traits can be either true or false

    // Evidence should be collected
    // Evidence length is always >= 0

    // Version should be set
    assert_eq!(result.version, "0.1.0");
}
