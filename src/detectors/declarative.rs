use crate::detectors::env_mapping::EnvMapping;
use crate::detectors::utils::{
    DetectionConfig, SelectionStrategy, basic_declarative_detection, check_generic_overrides,
};
use crate::detectors::{Detection, EnvSnapshot};
use crate::schema::Evidence;
use serde_json::json;

/// Base trait for declarative detectors that provides common functionality
///
/// This trait standardizes the detection pattern across all declarative detectors,
/// reducing code duplication and ensuring consistent behavior.
pub trait DeclarativeDetector {
    /// Get the mappings for this detector
    fn get_mappings() -> Vec<EnvMapping>;

    /// Get the detector type identifier (e.g., "agent", "ide", "ci")
    fn get_detector_type() -> &'static str;

    /// Get the context name to add when detection occurs (e.g., "agent", "ide", "ci")
    fn get_context_name() -> &'static str;

    /// Get the facet key for the detected ID (e.g., "agent_id", "ide_id", "ci_id")
    fn get_facet_key() -> &'static str;

    /// Whether this detector should generate evidence
    fn should_generate_evidence() -> bool {
        true
    }

    /// Get the supports list for evidence generation
    fn get_supports() -> Vec<String> {
        vec![
            Self::get_context_name().into(),
            Self::get_facet_key().into(),
        ]
    }

    /// Get the selection strategy for this detector
    fn get_selection_strategy() -> SelectionStrategy {
        SelectionStrategy::Confidence
    }

    /// Perform detection using the standard declarative pattern
    fn detect_with_mappings(&self, snap: &EnvSnapshot) -> (Option<String>, f32, Vec<Evidence>) {
        // Check for overrides first
        if let Some(override_result) = check_generic_overrides(snap, Self::get_detector_type()) {
            return override_result;
        }

        // Use standard detection logic
        let config = DetectionConfig {
            context_name: Self::get_context_name().to_string(),
            facet_key: Self::get_facet_key().to_string(),
            should_generate_evidence: Self::should_generate_evidence(),
            supports: Self::get_supports(),
        };

        basic_declarative_detection(
            &Self::get_mappings(),
            &snap.env_vars,
            &config,
            Self::get_selection_strategy(),
        )
    }

    /// Create a Detection object from the detection results
    fn create_detection(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();
        let (id, confidence, evidence) = self.detect_with_mappings(snap);

        if let Some(detected_id) = id {
            detection
                .contexts_add
                .push(Self::get_context_name().to_string());
            detection
                .facets_patch
                .insert(Self::get_facet_key().to_string(), json!(detected_id));
            detection.confidence = confidence;
            detection.evidence = evidence;
        }

        detection
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detectors::confidence::HIGH;
    use crate::detectors::env_mapping::{EnvIndicator, EnvMapping};
    use std::collections::HashMap;

    // Test implementation of DeclarativeDetector
    struct TestDetector;

    impl DeclarativeDetector for TestDetector {
        fn get_mappings() -> Vec<EnvMapping> {
            vec![EnvMapping {
                id: "test".to_string(),
                confidence: HIGH,
                indicators: vec![EnvIndicator {
                    key: "TEST_VAR".to_string(),
                    value: None,
                    required: false,
                    prefix: false,
                    contains: None,
                    priority: 1,
                }],
                facets: HashMap::from([("test_id".to_string(), "test".to_string())]),
                contexts: vec!["test".to_string()],
            }]
        }

        fn get_detector_type() -> &'static str {
            "test"
        }

        fn get_context_name() -> &'static str {
            "test"
        }

        fn get_facet_key() -> &'static str {
            "test_id"
        }
    }

    #[test]
    fn test_declarative_detector_basic_detection() {
        let detector = TestDetector;
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "1".to_string());
        let snap = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

        let detection = detector.create_detection(&snap);

        assert!(detection.contexts_add.contains(&"test".to_string()));
        assert_eq!(
            detection.facets_patch.get("test_id").unwrap(),
            &json!("test")
        );
        assert_eq!(detection.confidence, HIGH);
        assert!(!detection.evidence.is_empty());
    }

    #[test]
    fn test_declarative_detector_override() {
        let detector = TestDetector;
        let mut env_vars = HashMap::new();
        env_vars.insert("ENVSENSE_TEST".to_string(), "override".to_string());
        let snap = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

        let detection = detector.create_detection(&snap);

        assert!(detection.contexts_add.contains(&"test".to_string()));
        assert_eq!(
            detection.facets_patch.get("test_id").unwrap(),
            &json!("override")
        );
        assert_eq!(detection.confidence, HIGH);
    }

    #[test]
    fn test_declarative_detector_no_detection() {
        let detector = TestDetector;
        let env_vars = HashMap::new();
        let snap = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

        let detection = detector.create_detection(&snap);

        assert!(detection.contexts_add.is_empty());
        assert!(detection.facets_patch.is_empty());
        assert_eq!(detection.confidence, 0.0);
        assert!(detection.evidence.is_empty());
    }
}
