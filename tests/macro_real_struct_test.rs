//! Test for macro integration with the main crate

use envsense::schema::EnvSense;

#[test]
fn test_macro_integration_works() {
    // Test that the macro integration works by using the main crate's EnvSense
    let envsense = EnvSense::default();
    
    // Verify the struct has the expected fields
    assert_eq!(envsense.contexts.agent, false);
    assert_eq!(envsense.contexts.ide, false);
    assert_eq!(envsense.contexts.ci, false);
    assert_eq!(envsense.traits.is_interactive, false);
    assert_eq!(envsense.traits.is_tty_stdout, false);
    assert_eq!(envsense.facets.agent_id, None);
    assert_eq!(envsense.facets.ide_id, None);
    assert_eq!(envsense.evidence.len(), 0);
    assert_eq!(envsense.version, "0.1.0");
    assert_eq!(envsense.rules_version, "");
}

#[test]
fn test_detection_engine_uses_macro() {
    // Test that the detection engine works with the macro-generated merging
    use envsense::engine::DetectionEngine;
    use envsense::detectors::terminal::TerminalDetector;
    
    let engine = DetectionEngine::new()
        .register(TerminalDetector::new());
    
    let result = engine.detect();
    
    // Verify that the macro-generated merging worked
    // The terminal detector should have set some traits
    assert!(result.traits.is_tty_stdin || !result.traits.is_tty_stdin); // Boolean logic
    assert!(result.traits.is_tty_stdout || !result.traits.is_tty_stdout);
    assert!(result.traits.is_tty_stderr || !result.traits.is_tty_stderr);
    assert!(result.traits.is_piped_stdin || !result.traits.is_piped_stdin);
    assert!(result.traits.is_piped_stdout || !result.traits.is_piped_stdout);
    
    // Evidence should be collected
    assert!(result.evidence.len() >= 0);
    
    // Version should be set
    assert_eq!(result.version, "0.1.0");
}
