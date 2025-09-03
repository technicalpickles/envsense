//! Tests for nested trait structure macro functionality
//!
//! This module tests the DetectionMergerDerive macro's ability to handle
//! nested trait structures like NestedTraits, ensuring proper field mapping
//! and merging logic for the new schema structure.

use envsense_macros::{Detection, DetectionMerger, DetectionMergerDerive};
use serde_json::json;
use std::collections::HashMap;

// Import the new nested trait structures
use envsense::traits::{ColorLevel, NestedTraits, TerminalTraits};

/// Test struct using the new nested traits structure
#[derive(DetectionMergerDerive, Default, Debug, PartialEq, Clone)]
pub struct TestNestedStruct {
    pub contexts: Vec<String>,
    pub traits: NestedTraits,
    pub evidence: Vec<serde_json::Value>,
}

/// Test struct for backward compatibility with flat traits
#[derive(DetectionMergerDerive, Default, Debug, PartialEq, Clone)]
pub struct TestLegacyStruct {
    pub contexts: Vec<String>,
    pub traits: TerminalTraits, // Legacy flat structure
    pub evidence: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nested_agent_trait_merging() {
        let mut test_struct = TestNestedStruct::default();

        let mut traits_patch = HashMap::new();
        traits_patch.insert("agent.id".to_string(), json!("cursor"));

        let detections = vec![Detection {
            contexts_add: vec!["agent".to_string()],
            traits_patch,
            facets_patch: HashMap::new(),
            evidence: vec![],
            confidence: 1.0,
        }];

        test_struct.merge_detections(&detections);

        assert_eq!(test_struct.traits.agent.id, Some("cursor".to_string()));
        assert_eq!(test_struct.contexts, vec!["agent"]);
    }

    #[test]
    fn test_nested_ide_trait_merging() {
        let mut test_struct = TestNestedStruct::default();

        let mut traits_patch = HashMap::new();
        traits_patch.insert("ide.id".to_string(), json!("vscode"));

        let detections = vec![Detection {
            contexts_add: vec!["ide".to_string()],
            traits_patch,
            facets_patch: HashMap::new(),
            evidence: vec![],
            confidence: 1.0,
        }];

        test_struct.merge_detections(&detections);

        assert_eq!(test_struct.traits.ide.id, Some("vscode".to_string()));
        assert_eq!(test_struct.contexts, vec!["ide"]);
    }

    #[test]
    fn test_nested_terminal_trait_merging() {
        let mut test_struct = TestNestedStruct::default();

        let mut traits_patch = HashMap::new();
        traits_patch.insert("terminal.interactive".to_string(), json!(true));
        traits_patch.insert("terminal.stdin.tty".to_string(), json!(true));
        traits_patch.insert("terminal.stdout.piped".to_string(), json!(false));
        traits_patch.insert("terminal.color_level".to_string(), json!("truecolor"));
        traits_patch.insert("terminal.supports_hyperlinks".to_string(), json!(true));

        let detections = vec![Detection {
            contexts_add: vec![],
            traits_patch,
            facets_patch: HashMap::new(),
            evidence: vec![],
            confidence: 1.0,
        }];

        test_struct.merge_detections(&detections);

        assert_eq!(test_struct.traits.terminal.interactive, true);
        assert_eq!(test_struct.traits.terminal.stdin.tty, true);
        assert_eq!(test_struct.traits.terminal.stdout.piped, false);
        assert_eq!(
            test_struct.traits.terminal.color_level,
            ColorLevel::Truecolor
        );
        assert_eq!(test_struct.traits.terminal.supports_hyperlinks, true);
    }

    #[test]
    fn test_nested_ci_trait_merging() {
        let mut test_struct = TestNestedStruct::default();

        let mut traits_patch = HashMap::new();
        traits_patch.insert("ci.id".to_string(), json!("github"));
        traits_patch.insert("ci.vendor".to_string(), json!("github"));
        traits_patch.insert("ci.name".to_string(), json!("GitHub Actions"));
        traits_patch.insert("ci.is_pr".to_string(), json!(true));
        traits_patch.insert("ci.branch".to_string(), json!("main"));

        let detections = vec![Detection {
            contexts_add: vec!["ci".to_string()],
            traits_patch,
            facets_patch: HashMap::new(),
            evidence: vec![],
            confidence: 1.0,
        }];

        test_struct.merge_detections(&detections);

        assert_eq!(test_struct.traits.ci.id, Some("github".to_string()));
        assert_eq!(test_struct.traits.ci.vendor, Some("github".to_string()));
        assert_eq!(
            test_struct.traits.ci.name,
            Some("GitHub Actions".to_string())
        );
        assert_eq!(test_struct.traits.ci.is_pr, Some(true));
        assert_eq!(test_struct.traits.ci.branch, Some("main".to_string()));
        assert_eq!(test_struct.contexts, vec!["ci"]);
    }

    #[test]
    fn test_multiple_nested_traits_merging() {
        let mut test_struct = TestNestedStruct::default();

        let mut traits_patch = HashMap::new();
        traits_patch.insert("agent.id".to_string(), json!("cursor"));
        traits_patch.insert("ide.id".to_string(), json!("cursor"));
        traits_patch.insert("terminal.interactive".to_string(), json!(true));
        traits_patch.insert("ci.id".to_string(), json!("github"));

        let detections = vec![Detection {
            contexts_add: vec!["agent".to_string(), "ide".to_string(), "ci".to_string()],
            traits_patch,
            facets_patch: HashMap::new(),
            evidence: vec![],
            confidence: 1.0,
        }];

        test_struct.merge_detections(&detections);

        assert_eq!(test_struct.traits.agent.id, Some("cursor".to_string()));
        assert_eq!(test_struct.traits.ide.id, Some("cursor".to_string()));
        assert_eq!(test_struct.traits.terminal.interactive, true);
        assert_eq!(test_struct.traits.ci.id, Some("github".to_string()));
        assert_eq!(test_struct.contexts, vec!["agent", "ide", "ci"]);
    }

    #[test]
    fn test_backward_compatibility_flat_traits() {
        let mut test_struct = TestNestedStruct::default();

        // Test that flat trait keys still work for backward compatibility
        let mut traits_patch = HashMap::new();
        traits_patch.insert("is_interactive".to_string(), json!(true));
        traits_patch.insert("is_tty_stdin".to_string(), json!(true));
        traits_patch.insert("is_tty_stdout".to_string(), json!(false));
        traits_patch.insert("color_level".to_string(), json!("ansi256"));
        traits_patch.insert("supports_hyperlinks".to_string(), json!(false));

        let detections = vec![Detection {
            contexts_add: vec![],
            traits_patch,
            facets_patch: HashMap::new(),
            evidence: vec![],
            confidence: 1.0,
        }];

        test_struct.merge_detections(&detections);

        assert_eq!(test_struct.traits.terminal.interactive, true);
        assert_eq!(test_struct.traits.terminal.stdin.tty, true);
        assert_eq!(test_struct.traits.terminal.stdout.tty, false);
        assert_eq!(test_struct.traits.terminal.color_level, ColorLevel::Ansi256);
        assert_eq!(test_struct.traits.terminal.supports_hyperlinks, false);
    }

    #[test]
    fn test_nested_vs_flat_precedence() {
        let mut test_struct = TestNestedStruct::default();

        // Test that nested keys take precedence over flat keys
        let mut traits_patch = HashMap::new();
        traits_patch.insert("is_interactive".to_string(), json!(false)); // Flat key
        traits_patch.insert("terminal.interactive".to_string(), json!(true)); // Nested key (should win)

        let detections = vec![Detection {
            contexts_add: vec![],
            traits_patch,
            facets_patch: HashMap::new(),
            evidence: vec![],
            confidence: 1.0,
        }];

        test_struct.merge_detections(&detections);

        // Nested key should take precedence (processed after flat key)
        assert_eq!(test_struct.traits.terminal.interactive, true);
    }

    #[test]
    fn test_legacy_struct_compatibility() {
        let mut test_struct = TestLegacyStruct::default();

        let mut traits_patch = HashMap::new();
        traits_patch.insert("is_interactive".to_string(), json!(true));
        traits_patch.insert("color_level".to_string(), json!("truecolor"));

        let detections = vec![Detection {
            contexts_add: vec![],
            traits_patch,
            facets_patch: HashMap::new(),
            evidence: vec![],
            confidence: 1.0,
        }];

        test_struct.merge_detections(&detections);

        assert_eq!(test_struct.traits.interactive, true);
        assert_eq!(test_struct.traits.color_level, ColorLevel::Truecolor);
    }

    #[test]
    fn test_evidence_merging_with_nested_traits() {
        let mut test_struct = TestNestedStruct::default();

        let evidence_value = json!({
            "signal": "TERM_PROGRAM",
            "value": "cursor",
            "confidence": 0.9
        });

        let detections = vec![Detection {
            contexts_add: vec![],
            traits_patch: HashMap::new(),
            facets_patch: HashMap::new(),
            evidence: vec![evidence_value.clone()],
            confidence: 1.0,
        }];

        test_struct.merge_detections(&detections);

        assert_eq!(test_struct.evidence.len(), 1);
        // Note: Evidence merging depends on the Evidence struct implementation
        // This test ensures the macro doesn't break evidence handling
    }

    #[test]
    fn test_empty_detection_handling() {
        let mut test_struct = TestNestedStruct::default();
        let original = test_struct.clone();

        let detections = vec![Detection {
            contexts_add: vec![],
            traits_patch: HashMap::new(),
            facets_patch: HashMap::new(),
            evidence: vec![],
            confidence: 1.0,
        }];

        test_struct.merge_detections(&detections);

        // Should remain unchanged
        assert_eq!(test_struct, original);
    }

    #[test]
    fn test_invalid_enum_values() {
        let mut test_struct = TestNestedStruct::default();

        let mut traits_patch = HashMap::new();
        traits_patch.insert("terminal.color_level".to_string(), json!("invalid_color"));

        let detections = vec![Detection {
            contexts_add: vec![],
            traits_patch,
            facets_patch: HashMap::new(),
            evidence: vec![],
            confidence: 1.0,
        }];

        test_struct.merge_detections(&detections);

        // Should default to None for invalid enum values
        assert_eq!(test_struct.traits.terminal.color_level, ColorLevel::None);
    }

    #[test]
    fn test_multiple_detections_merging() {
        let mut test_struct = TestNestedStruct::default();

        let detections = vec![
            Detection {
                contexts_add: vec!["agent".to_string()],
                traits_patch: {
                    let mut patch = HashMap::new();
                    patch.insert("agent.id".to_string(), json!("cursor"));
                    patch
                },
                facets_patch: HashMap::new(),
                evidence: vec![],
                confidence: 1.0,
            },
            Detection {
                contexts_add: vec!["ci".to_string()],
                traits_patch: {
                    let mut patch = HashMap::new();
                    patch.insert("ci.id".to_string(), json!("github"));
                    patch.insert("terminal.interactive".to_string(), json!(false));
                    patch
                },
                facets_patch: HashMap::new(),
                evidence: vec![],
                confidence: 0.8,
            },
        ];

        test_struct.merge_detections(&detections);

        assert_eq!(test_struct.traits.agent.id, Some("cursor".to_string()));
        assert_eq!(test_struct.traits.ci.id, Some("github".to_string()));
        assert_eq!(test_struct.traits.terminal.interactive, false);
        assert_eq!(test_struct.contexts, vec!["agent", "ci"]);
    }
}
