use envsense::detectors::DeclarativeAgentDetector;
use envsense::detectors::{Detector, EnvSnapshot};
use envsense::schema::EnvSense;
use std::collections::HashMap;

#[test]
fn debug_host_detection() {
    let detector = DeclarativeAgentDetector::new();

    // Create a test environment with Replit
    let mut env_vars = HashMap::new();
    env_vars.insert("REPLIT_USER".to_string(), "josh".to_string());

    let snapshot = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

    // Run detection
    let detection = detector.detect(&snapshot);

    println!("Detection result:");
    println!("Contexts: {:?}", detection.contexts_add);
    println!("Facets: {:?}", detection.facets_patch);
    println!("Evidence count: {}", detection.evidence.len());

    // Check for host facet
    if let Some(host) = detection.facets_patch.get("host") {
        println!("Host detected: {}", host);
    } else {
        println!("No host detected!");
    }

    // Check for host evidence
    for evidence in &detection.evidence {
        if evidence.supports.contains(&"host".to_string()) {
            println!("Host evidence: {} = {:?}", evidence.key, evidence.value);
        }
    }

    // Test the full EnvSense detection
    let envsense = EnvSense::detect();
    println!("Full EnvSense facets: {:?}", envsense.facets);
    println!("Host field: {:?}", envsense.facets.host);

    // The test should fail if host is not detected
    assert!(
        detection.facets_patch.contains_key("host"),
        "Host should be detected"
    );
    assert!(
        envsense.facets.host.is_some(),
        "Host should be set in Facets struct"
    );
}
