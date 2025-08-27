use crate::detectors::env_mapping::get_ide_mappings;
use crate::detectors::{Detection, Detector, EnvSnapshot, confidence::HIGH};
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

        // Special handling for VS Code variants
        if let Some(term_program) = snap.get_env("TERM_PROGRAM")
            && term_program == "vscode"
        {
            // Check for Cursor first (highest priority)
            if snap.get_env("CURSOR_TRACE_ID").is_some() {
                ide_id = Some("cursor".to_string());
                confidence = HIGH;
                evidence.push(
                    Evidence::env_var("TERM_PROGRAM", term_program)
                        .with_supports(vec!["ide".into(), "ide_id".into()])
                        .with_confidence(HIGH),
                );
                evidence.push(
                    Evidence::env_presence("CURSOR_TRACE_ID")
                        .with_supports(vec!["ide".into(), "ide_id".into()])
                        .with_confidence(HIGH),
                );
            }
            // Check for VS Code Insiders
            else if let Some(version) = snap.get_env("TERM_PROGRAM_VERSION") {
                if version.to_lowercase().contains("insider") {
                    ide_id = Some("vscode-insiders".to_string());
                    confidence = HIGH;
                    evidence.push(
                        Evidence::env_var("TERM_PROGRAM", term_program)
                            .with_supports(vec!["ide".into(), "ide_id".into()])
                            .with_confidence(HIGH),
                    );
                    evidence.push(
                        Evidence::env_var("TERM_PROGRAM_VERSION", version)
                            .with_supports(vec!["ide".into(), "ide_id".into()])
                            .with_confidence(HIGH),
                    );
                } else {
                    // Regular VS Code
                    ide_id = Some("vscode".to_string());
                    confidence = HIGH;
                    evidence.push(
                        Evidence::env_var("TERM_PROGRAM", term_program)
                            .with_supports(vec!["ide".into(), "ide_id".into()])
                            .with_confidence(HIGH),
                    );
                    if let Some(version) = snap.get_env("TERM_PROGRAM_VERSION") {
                        evidence.push(
                            Evidence::env_var("TERM_PROGRAM_VERSION", version)
                                .with_supports(vec!["ide".into(), "ide_id".into()])
                                .with_confidence(HIGH),
                        );
                    }
                }
            } else {
                // Regular VS Code without version
                ide_id = Some("vscode".to_string());
                confidence = HIGH;
                evidence.push(
                    Evidence::env_var("TERM_PROGRAM", term_program)
                        .with_supports(vec!["ide".into(), "ide_id".into()])
                        .with_confidence(HIGH),
                );
            }
        }

        // If no VS Code variant detected, try other IDE mappings
        if ide_id.is_none() {
            for mapping in &mappings {
                if mapping.matches(&snap.env_vars) && mapping.confidence > confidence {
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
                    break; // Take the first (highest confidence) match
                }
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
        assert_eq!(detection.evidence.len(), 2);
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
