use crate::detectors::{Detection, Detector, EnvSnapshot, confidence::HIGH};
use crate::schema::Evidence;
use serde_json::json;

pub struct IdeDetector;

impl IdeDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Detector for IdeDetector {
    fn name(&self) -> &'static str {
        "ide"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        if let Some(term_program) = snap.get_env("TERM_PROGRAM")
            && term_program == "vscode"
        {
            detection.contexts_add.push("ide".to_string());
            detection.confidence = HIGH; // Direct env var match

            // Add evidence for IDE context
            detection.evidence.push(
                Evidence::env_var("TERM_PROGRAM", term_program)
                    .with_supports(vec!["ide".into()])
                    .with_confidence(HIGH)
            );

            // Detect specific IDE variant
            if snap.get_env("CURSOR_TRACE_ID").is_some() {
                detection
                    .facets_patch
                    .insert("ide_id".to_string(), json!("cursor"));
                detection.evidence.push(
                    Evidence::env_presence("CURSOR_TRACE_ID")
                        .with_supports(vec!["ide_id".into()])
                        .with_confidence(HIGH) // Direct env var presence
                );
            } else if let Some(version) = snap.get_env("TERM_PROGRAM_VERSION") {
                let ide_id = if version.to_lowercase().contains("insider") {
                    "vscode-insiders"
                } else {
                    "vscode"
                };
                detection
                    .facets_patch
                    .insert("ide_id".to_string(), json!(ide_id));
                detection.evidence.push(
                    Evidence::env_var("TERM_PROGRAM_VERSION", version)
                        .with_supports(vec!["ide_id".into()])
                        .with_confidence(HIGH) // Direct env var match
                );
            }
        }

        detection
    }
}

impl Default for IdeDetector {
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

        EnvSnapshot {
            env_vars: env_map,
            is_tty_stdin: false,
            is_tty_stdout: false,
            is_tty_stderr: false,
        }
    }

    #[test]
    fn detects_vscode() {
        let detector = IdeDetector::new();
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
        let detector = IdeDetector::new();
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
    fn detects_cursor() {
        let detector = IdeDetector::new();
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
        let detector = IdeDetector::new();
        let snapshot = create_env_snapshot(vec![]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.is_empty());
        assert!(detection.facets_patch.is_empty());
        assert!(detection.evidence.is_empty());
        assert_eq!(detection.confidence, 0.0);
    }

    #[test]
    fn no_detection_with_different_term_program() {
        let detector = IdeDetector::new();
        let snapshot = create_env_snapshot(vec![("TERM_PROGRAM", "iTerm.app")]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.is_empty());
        assert!(detection.facets_patch.is_empty());
        assert!(detection.evidence.is_empty());
        assert_eq!(detection.confidence, 0.0);
    }
}
