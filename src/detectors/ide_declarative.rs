use crate::detectors::env_mapping::{EnvMapping, get_ide_mappings};
use crate::detectors::{Detection, Detector, EnvSnapshot};
use crate::schema::Evidence;
use serde_json::json;

pub struct DeclarativeIdeDetector;

impl DeclarativeIdeDetector {
    pub fn new() -> Self {
        Self
    }

    fn detect_ide(&self, snap: &EnvSnapshot) -> (Option<String>, f32, Vec<Evidence>) {
        let mappings = get_ide_mappings();
        let mut ide_id = None;
        let mut confidence = 0.0;
        let mut evidence = Vec::new();

        // Find the highest priority matching mapping
        let mut best_mapping: Option<&EnvMapping> = None;
        let mut best_priority = 0;

        for mapping in &mappings {
            if mapping.matches(&snap.env_vars) {
                let mapping_priority = mapping.get_highest_priority();
                if mapping_priority > best_priority {
                    best_mapping = Some(mapping);
                    best_priority = mapping_priority;
                }
            }
        }

        if let Some(mapping) = best_mapping {
            ide_id = mapping.facets.get("ide_id").cloned();
            confidence = mapping.confidence;

            // Add evidence for this detection
            for (key, value) in mapping.get_evidence(&snap.env_vars) {
                let evidence_item = if let Some(val) = value {
                    Evidence::env_var(key, val)
                } else {
                    Evidence::env_presence(key)
                };
                evidence.push(
                    evidence_item
                        .with_supports(vec!["ide".into(), "ide_id".into()])
                        .with_confidence(mapping.confidence),
                );
            }
        }

        (ide_id, confidence, evidence)
    }
}

impl Detector for DeclarativeIdeDetector {
    fn name(&self) -> &'static str {
        "ide-declarative"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        let (ide_id, confidence, evidence) = self.detect_ide(snap);

        if let Some(id) = ide_id {
            detection.contexts_add.push("ide".to_string());
            detection
                .facets_patch
                .insert("ide_id".to_string(), json!(id));
            detection.confidence = confidence;
            detection.evidence = evidence;
        }

        detection
    }
}

impl Default for DeclarativeIdeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_env_snapshot(env_vars: Vec<(&str, &str)>) -> EnvSnapshot {
        let mut env_map = HashMap::new();
        for (k, v) in env_vars {
            env_map.insert(k.to_string(), v.to_string());
        }

        EnvSnapshot::with_mock_tty(env_map, false, false, false)
    }

    #[test]
    fn detects_vscode() {
        let detector = DeclarativeIdeDetector::new();
        let snapshot = create_env_snapshot(vec![
            ("TERM_PROGRAM", "vscode"),
            ("TERM_PROGRAM_VERSION", "1.85.0"),
        ]);

        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ide"]);
        assert_eq!(
            detection.facets_patch.get("ide_id").unwrap(),
            &json!("vscode")
        );
        assert_eq!(detection.evidence.len(), 1);
        assert_eq!(detection.confidence, HIGH);
    }

    #[test]
    fn detects_vscode_insiders() {
        let detector = DeclarativeIdeDetector::new();
        let snapshot = create_env_snapshot(vec![
            ("TERM_PROGRAM", "vscode"),
            ("TERM_PROGRAM_VERSION", "1.86.0-insider"),
        ]);

        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ide"]);
        assert_eq!(
            detection.facets_patch.get("ide_id").unwrap(),
            &json!("vscode-insiders")
        );
        assert_eq!(detection.evidence.len(), 2);
        assert_eq!(detection.confidence, HIGH);
    }

    #[test]
    fn detects_cursor_ide() {
        let detector = DeclarativeIdeDetector::new();
        let snapshot = create_env_snapshot(vec![
            ("TERM_PROGRAM", "vscode"),
            ("CURSOR_TRACE_ID", "abc123"),
        ]);

        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ide"]);
        assert_eq!(
            detection.facets_patch.get("ide_id").unwrap(),
            &json!("cursor")
        );
        assert_eq!(detection.evidence.len(), 2);
        assert_eq!(detection.confidence, HIGH);
    }

    #[test]
    fn no_detection_without_vscode() {
        let detector = DeclarativeIdeDetector::new();
        let snapshot = create_env_snapshot(vec![]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.is_empty());
        assert!(detection.facets_patch.is_empty());
        assert!(detection.evidence.is_empty());
        assert_eq!(detection.confidence, 0.0);
    }

    #[test]
    fn no_detection_with_different_term_program() {
        let detector = DeclarativeIdeDetector::new();
        let snapshot = create_env_snapshot(vec![("TERM_PROGRAM", "iTerm.app")]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.is_empty());
        assert!(detection.facets_patch.is_empty());
        assert!(detection.evidence.is_empty());
        assert_eq!(detection.confidence, 0.0);
    }
}
