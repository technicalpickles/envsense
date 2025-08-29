use crate::detectors::env_mapping::EnvMapping;
use crate::detectors::{EnvSnapshot, confidence::HIGH};
use crate::schema::Evidence;
use std::collections::HashMap;

/// Generate evidence from a mapping's indicators
pub fn generate_evidence_from_mapping(
    mapping: &EnvMapping,
    env_vars: &HashMap<String, String>,
    supports: Vec<String>,
) -> Vec<Evidence> {
    let mut evidence = Vec::new();

    for (key, value) in mapping.get_evidence(env_vars) {
        let evidence_item = if let Some(val) = value {
            Evidence::env_var(key, val)
        } else {
            Evidence::env_presence(key)
        };
        evidence.push(
            evidence_item
                .with_supports(supports.clone())
                .with_confidence(mapping.confidence),
        );
    }

    evidence
}

/// Find the best mapping by confidence (highest confidence wins)
pub fn find_best_mapping_by_confidence<'a>(
    mappings: &'a [EnvMapping],
    env_vars: &HashMap<String, String>,
) -> Option<&'a EnvMapping> {
    let mut best_mapping = None;
    let mut best_confidence = 0.0;

    for mapping in mappings {
        if mapping.matches(env_vars) && mapping.confidence > best_confidence {
            best_mapping = Some(mapping);
            best_confidence = mapping.confidence;
        }
    }

    best_mapping
}

/// Find the best mapping by priority (highest priority wins)
pub fn find_best_mapping_by_priority<'a>(
    mappings: &'a [EnvMapping],
    env_vars: &HashMap<String, String>,
) -> Option<&'a EnvMapping> {
    let mut best_mapping = None;
    let mut best_priority = 0;

    for mapping in mappings {
        if mapping.matches(env_vars) {
            let mapping_priority = mapping.get_highest_priority();
            if mapping_priority > best_priority {
                best_mapping = Some(mapping);
                best_priority = mapping_priority;
            }
        }
    }

    best_mapping
}

/// Selection strategy for mapping selection
#[derive(Debug, Clone, Copy)]
pub enum SelectionStrategy {
    Confidence,
    Priority,
}

/// Configuration for basic declarative detection
pub struct DetectionConfig {
    pub context_name: String,
    pub facet_key: String,
    pub should_generate_evidence: bool,
    pub supports: Vec<String>,
}

/// Basic declarative detection pattern
pub fn basic_declarative_detection(
    mappings: &[EnvMapping],
    env_vars: &HashMap<String, String>,
    config: &DetectionConfig,
    selection_strategy: SelectionStrategy,
) -> (Option<String>, f32, Vec<Evidence>) {
    let best_mapping = match selection_strategy {
        SelectionStrategy::Confidence => find_best_mapping_by_confidence(mappings, env_vars),
        SelectionStrategy::Priority => find_best_mapping_by_priority(mappings, env_vars),
    };

    if let Some(mapping) = best_mapping {
        let id = mapping.facets.get(&config.facet_key).cloned();
        let confidence = mapping.confidence;
        let evidence = if config.should_generate_evidence {
            generate_evidence_from_mapping(mapping, env_vars, config.supports.clone())
        } else {
            Vec::new()
        };

        (id, confidence, evidence)
    } else {
        (None, 0.0, Vec::new())
    }
}

/// Check for generic overrides for any detector type
pub fn check_generic_overrides(
    snap: &EnvSnapshot,
    detector_type: &str,
) -> Option<(Option<String>, f32, Vec<Evidence>)> {
    let override_key = format!("ENVSENSE_{}", detector_type.to_uppercase());
    let assume_key = format!(
        "ENVSENSE_ASSUME_{}",
        match detector_type {
            "agent" => "HUMAN",
            "ide" => "TERMINAL",
            "ci" => "LOCAL",
            _ => return None,
        }
    );

    // Check for assume override (disable detection)
    if snap.get_env(&assume_key).map(|v| v == "1").unwrap_or(false) {
        return Some((None, 0.0, vec![]));
    }

    // Check for direct override
    if let Some(override_value) = snap.get_env(&override_key) {
        if override_value == "none" {
            return Some((None, 0.0, vec![]));
        } else {
            let evidence = vec![
                Evidence::env_var(&override_key, override_value)
                    .with_supports(vec![detector_type.into(), format!("{}_id", detector_type)])
                    .with_confidence(HIGH),
            ];
            return Some((Some(override_value.clone()), HIGH, evidence));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detectors::confidence::HIGH;

    fn create_test_mapping(id: &str, confidence: f32, priority: u8) -> EnvMapping {
        EnvMapping {
            id: id.to_string(),
            confidence,
            indicators: vec![crate::detectors::env_mapping::EnvIndicator {
                key: format!("TEST_{}", id.to_uppercase()),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority,
            }],
            facets: HashMap::from([("test_id".to_string(), id.to_string())]),
            contexts: vec!["test".to_string()],
        }
    }

    #[test]
    fn test_find_best_mapping_by_confidence() {
        let mappings = vec![
            create_test_mapping("low", 0.5, 1),
            create_test_mapping("high", 0.9, 1),
            create_test_mapping("medium", 0.7, 1),
        ];

        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_LOW".to_string(), "1".to_string());
        env_vars.insert("TEST_HIGH".to_string(), "1".to_string());
        env_vars.insert("TEST_MEDIUM".to_string(), "1".to_string());

        let best = find_best_mapping_by_confidence(&mappings, &env_vars);
        assert_eq!(best.unwrap().id, "high");
    }

    #[test]
    fn test_find_best_mapping_by_priority() {
        let mappings = vec![
            create_test_mapping("low", 0.9, 1),
            create_test_mapping("high", 0.5, 3),
            create_test_mapping("medium", 0.7, 2),
        ];

        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_LOW".to_string(), "1".to_string());
        env_vars.insert("TEST_HIGH".to_string(), "1".to_string());
        env_vars.insert("TEST_MEDIUM".to_string(), "1".to_string());

        let best = find_best_mapping_by_priority(&mappings, &env_vars);
        assert_eq!(best.unwrap().id, "high");
    }

    #[test]
    fn test_generate_evidence_from_mapping() {
        let mapping = create_test_mapping("test", HIGH, 1);
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_TEST".to_string(), "value".to_string());

        let evidence = generate_evidence_from_mapping(
            &mapping,
            &env_vars,
            vec!["test".to_string(), "test_id".to_string()],
        );

        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].confidence, HIGH);
    }

    #[test]
    fn test_basic_declarative_detection() {
        let mappings = vec![create_test_mapping("test", HIGH, 1)];
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_TEST".to_string(), "value".to_string());

        let config = DetectionConfig {
            context_name: "test".to_string(),
            facet_key: "test_id".to_string(),
            should_generate_evidence: true,
            supports: vec!["test".to_string(), "test_id".to_string()],
        };

        let (id, confidence, evidence) =
            basic_declarative_detection(&mappings, &env_vars, &config, SelectionStrategy::Priority);

        assert_eq!(id, Some("test".to_string()));
        assert_eq!(confidence, HIGH);
        assert_eq!(evidence.len(), 1);
    }

    #[test]
    fn test_check_generic_overrides_agent() {
        let mut env_vars = HashMap::new();
        env_vars.insert("ENVSENSE_AGENT".to_string(), "custom-agent".to_string());
        let snap = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

        let result = check_generic_overrides(&snap, "agent");
        let (id, confidence, evidence) = result.unwrap();
        assert_eq!(id, Some("custom-agent".to_string()));
        assert_eq!(confidence, HIGH);
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].key, "ENVSENSE_AGENT");
    }

    #[test]
    fn test_check_generic_overrides_ide() {
        let mut env_vars = HashMap::new();
        env_vars.insert("ENVSENSE_IDE".to_string(), "custom-editor".to_string());
        let snap = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

        let result = check_generic_overrides(&snap, "ide");
        let (id, confidence, evidence) = result.unwrap();
        assert_eq!(id, Some("custom-editor".to_string()));
        assert_eq!(confidence, HIGH);
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].key, "ENVSENSE_IDE");
    }

    #[test]
    fn test_check_generic_overrides_ci() {
        let mut env_vars = HashMap::new();
        env_vars.insert("ENVSENSE_CI".to_string(), "custom-ci".to_string());
        let snap = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

        let result = check_generic_overrides(&snap, "ci");
        let (id, confidence, evidence) = result.unwrap();
        assert_eq!(id, Some("custom-ci".to_string()));
        assert_eq!(confidence, HIGH);
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].key, "ENVSENSE_CI");
    }

    #[test]
    fn test_check_generic_overrides_none() {
        let mut env_vars = HashMap::new();
        env_vars.insert("ENVSENSE_AGENT".to_string(), "none".to_string());
        let snap = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

        let result = check_generic_overrides(&snap, "agent");
        assert_eq!(result, Some((None, 0.0, vec![])));
    }

    #[test]
    fn test_check_generic_overrides_assume() {
        let mut env_vars = HashMap::new();
        env_vars.insert("ENVSENSE_ASSUME_HUMAN".to_string(), "1".to_string());
        let snap = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

        let result = check_generic_overrides(&snap, "agent");
        assert_eq!(result, Some((None, 0.0, vec![])));
    }

    #[test]
    fn test_check_generic_overrides_no_override() {
        let env_vars = HashMap::new();
        let snap = EnvSnapshot::with_mock_tty(env_vars, false, false, false);

        let result = check_generic_overrides(&snap, "agent");
        assert_eq!(result, None);
    }
}
