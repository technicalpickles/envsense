use envsense::schema::EnvSense;
use envsense::detectors::confidence::{HIGH, MEDIUM, LOW, TERMINAL};
use envsense::schema::{Evidence, Signal};

#[test]
fn test_confidence_constants() {
    // Verify constants are in valid range
    assert!(HIGH >= 0.0 && HIGH <= 1.0);
    assert!(MEDIUM >= 0.0 && MEDIUM <= 1.0);
    assert!(LOW >= 0.0 && LOW <= 1.0);
    assert!(TERMINAL >= 0.0 && TERMINAL <= 1.0);
    
    // Verify relative ordering
    assert!(HIGH >= MEDIUM);
    assert!(MEDIUM >= LOW);
    assert_eq!(HIGH, TERMINAL); // Both are 1.0
    
    // Verify specific values
    assert_eq!(HIGH, 1.0);
    assert_eq!(MEDIUM, 0.8);
    assert_eq!(LOW, 0.6);
    assert_eq!(TERMINAL, 1.0);
}

#[test]
fn test_detection_confidence_values() {
    let result = EnvSense::detect();
    
    // Verify all detections have appropriate confidence levels
    for evidence in &result.evidence {
        match evidence.signal {
            Signal::Env => {
                if evidence.value.is_some() {
                    assert_eq!(evidence.confidence, HIGH, 
                        "Direct env var should have HIGH confidence: {}", evidence.key);
                } else {
                    // Env presence should be MEDIUM or HIGH (if overridden for direct indicators)
                    assert!(evidence.confidence >= MEDIUM && evidence.confidence <= HIGH,
                        "Env presence should have MEDIUM or HIGH confidence: {} = {}", evidence.key, evidence.confidence);
                }
            }
            Signal::Tty => {
                assert_eq!(evidence.confidence, TERMINAL, 
                    "TTY detection should have TERMINAL confidence: {}", evidence.key);
            }
            _ => {
                assert!(evidence.confidence >= LOW && evidence.confidence <= HIGH,
                    "Confidence should be in valid range: {} = {}", evidence.key, evidence.confidence);
            }
        }
    }
}

#[test]
fn test_evidence_constructors_use_confidence_constants() {
    // Test that evidence constructors use the correct confidence constants
    
    // env_var should use HIGH confidence
    let env_evidence = Evidence::env_var("TEST_VAR", "test_value");
    assert_eq!(env_evidence.confidence, HIGH);
    assert_eq!(env_evidence.signal, Signal::Env);
    assert_eq!(env_evidence.key, "TEST_VAR");
    assert_eq!(env_evidence.value, Some("test_value".to_string()));
    
    // env_presence should use MEDIUM confidence
    let presence_evidence = Evidence::env_presence("TEST_VAR");
    assert_eq!(presence_evidence.confidence, MEDIUM);
    assert_eq!(presence_evidence.signal, Signal::Env);
    assert_eq!(presence_evidence.key, "TEST_VAR");
    assert_eq!(presence_evidence.value, None);
    
    // tty_trait should use TERMINAL confidence
    let tty_evidence = Evidence::tty_trait("is_tty_stdin", true);
    assert_eq!(tty_evidence.confidence, TERMINAL);
    assert_eq!(tty_evidence.signal, Signal::Tty);
    assert_eq!(tty_evidence.key, "is_tty_stdin");
    assert_eq!(tty_evidence.value, Some("true".to_string()));
}

#[test]
fn test_confidence_override() {
    // Test that with_confidence can override the default confidence
    let evidence = Evidence::env_var("TEST_VAR", "test_value")
        .with_confidence(LOW);
    
    assert_eq!(evidence.confidence, LOW);
    assert_eq!(evidence.signal, Signal::Env);
    assert_eq!(evidence.key, "TEST_VAR");
}

#[test]
fn test_confidence_documentation() {
    // Verify that confidence levels are well-documented
    let result = EnvSense::detect();
    
    // Check that evidence has appropriate confidence based on signal type
    for evidence in &result.evidence {
        match evidence.signal {
            Signal::Env => {
                if evidence.value.is_some() {
                    // Direct env var match should be HIGH
                    assert!(evidence.confidence >= HIGH, 
                        "Direct env var {} should have HIGH confidence, got {}", 
                        evidence.key, evidence.confidence);
                } else {
                    // Env presence should be MEDIUM or HIGH (if overridden)
                    assert!(evidence.confidence >= MEDIUM, 
                        "Env presence {} should have MEDIUM or HIGH confidence, got {}", 
                        evidence.key, evidence.confidence);
                }
            }
            Signal::Tty => {
                // TTY detection should always be TERMINAL
                assert_eq!(evidence.confidence, TERMINAL, 
                    "TTY detection {} should have TERMINAL confidence", evidence.key);
            }
            _ => {
                // Other signals should be in valid range
                assert!(evidence.confidence >= LOW && evidence.confidence <= HIGH,
                    "Signal {:?} should have confidence in [LOW, HIGH] range", evidence.signal);
            }
        }
    }
}
