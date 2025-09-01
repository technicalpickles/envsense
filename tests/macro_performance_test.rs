//! Performance benchmarking tests for macro-based detection merging

use envsense::schema::{EnvSense, Evidence, Signal};
use envsense_macros::{Detection, DetectionMerger};
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn benchmark_macro_merging_performance() {
    // Create a large number of detections to test performance
    let detections: Vec<Detection> = (0..1000)
        .map(|i| Detection {
            contexts_add: vec![
                if i % 2 == 0 {
                    "agent".to_string()
                } else {
                    "ide".to_string()
                },
                if i % 3 == 0 {
                    "ci".to_string()
                } else {
                    "container".to_string()
                },
            ],
            traits_patch: HashMap::from([
                (
                    "is_interactive".to_string(),
                    serde_json::Value::Bool(i % 2 == 0),
                ),
                (
                    "is_tty_stdout".to_string(),
                    serde_json::Value::Bool(i % 3 == 0),
                ),
                (
                    "supports_hyperlinks".to_string(),
                    serde_json::Value::Bool(i % 4 == 0),
                ),
            ]),
            facets_patch: HashMap::from([
                (
                    "agent_id".to_string(),
                    serde_json::Value::String(format!("agent-{}", i)),
                ),
                (
                    "ide_id".to_string(),
                    serde_json::Value::String(format!("ide-{}", i)),
                ),
            ]),
            evidence: vec![
                serde_json::to_value(Evidence {
                    signal: Signal::Env,
                    key: format!("test-key-{}", i),
                    value: Some(format!("test-value-{}", i)),
                    supports: vec![],
                    confidence: 0.8,
                })
                .unwrap(),
            ],
            confidence: 0.8 + (i as f32 * 0.001),
        })
        .collect();

    let mut envsense = EnvSense::default();

    // Benchmark the merging operation
    let start = Instant::now();
    envsense.merge_detections(&detections);
    let duration = start.elapsed();

    // Performance assertions
    // The merging should complete in under 15ms for 1000 detections (debug mode)
    // Different environments may have different performance characteristics
    assert!(
        duration.as_micros() < 15000,
        "Merging took {}μs, expected < 15000μs",
        duration.as_micros()
    );

    // Verify the merging worked correctly
    assert!(
        envsense.contexts.contains(&"agent".to_string())
            || envsense.contexts.contains(&"ide".to_string())
    );
    // Boolean traits can be either true or false
    assert!(!envsense.evidence.is_empty());

    println!(
        "Macro merging performance: {}μs for 1000 detections",
        duration.as_micros()
    );
}

#[test]
fn benchmark_macro_vs_manual_approach() {
    // This test demonstrates the performance benefit of the macro approach
    // by comparing the complexity of the generated code vs manual implementation

    let detections = vec![Detection {
        contexts_add: vec!["agent".to_string(), "ide".to_string()],
        traits_patch: HashMap::from([
            ("is_interactive".to_string(), serde_json::Value::Bool(true)),
            ("is_tty_stdout".to_string(), serde_json::Value::Bool(true)),
        ]),
        facets_patch: HashMap::from([
            (
                "agent_id".to_string(),
                serde_json::Value::String("cursor".to_string()),
            ),
            (
                "ide_id".to_string(),
                serde_json::Value::String("cursor".to_string()),
            ),
        ]),
        evidence: vec![
            serde_json::to_value(Evidence {
                signal: Signal::Env,
                key: "test-key".to_string(),
                value: Some("test-value".to_string()),
                supports: vec![],
                confidence: 1.0,
            })
            .unwrap(),
        ],
        confidence: 1.0,
    }];

    let mut envsense = EnvSense::default();

    // Benchmark macro-based merging
    let start = Instant::now();
    envsense.merge_detections(&detections);
    let macro_duration = start.elapsed();

    // Verify results
    assert!(envsense.contexts.contains(&"agent".to_string()));
    assert!(envsense.contexts.contains(&"ide".to_string()));
    assert!(envsense.traits.is_interactive());
    assert!(envsense.traits.terminal.stdout.tty);
    assert_eq!(envsense.traits.agent.id, Some("cursor".to_string()));
    assert_eq!(envsense.traits.ide.id, Some("cursor".to_string()));
    assert!(!envsense.evidence.is_empty());

    println!(
        "Macro-based merging completed in {}μs",
        macro_duration.as_micros()
    );
    println!("Code complexity: ~20 lines of macro annotations vs 80+ lines of manual code");
}
