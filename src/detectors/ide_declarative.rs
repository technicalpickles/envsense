use crate::detectors::declarative::DeclarativeDetector;
use crate::detectors::env_mapping::get_ide_mappings;
use crate::detectors::utils::SelectionStrategy;
use crate::detectors::{Detection, Detector, EnvSnapshot};

pub struct DeclarativeIdeDetector;

impl DeclarativeIdeDetector {
    pub fn new() -> Self {
        Self
    }
}

impl DeclarativeDetector for DeclarativeIdeDetector {
    fn get_mappings() -> Vec<crate::detectors::env_mapping::EnvMapping> {
        get_ide_mappings()
    }

    fn get_detector_type() -> &'static str {
        "ide"
    }

    fn get_context_name() -> &'static str {
        "ide"
    }

    fn get_facet_key() -> &'static str {
        "ide_id"
    }

    fn get_selection_strategy() -> SelectionStrategy {
        SelectionStrategy::Priority
    }
}

impl Detector for DeclarativeIdeDetector {
    fn name(&self) -> &'static str {
        "ide-declarative"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        self.create_detection(snap)
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
    use crate::detectors::confidence::HIGH;
    use serde_json::json;
    use std::collections::HashMap;

    use crate::detectors::test_utils::create_env_snapshot;

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

    #[test]
    fn respects_override_force_ide() {
        let detector = DeclarativeIdeDetector::new();
        let snapshot = create_env_snapshot(vec![("ENVSENSE_IDE", "custom-editor")]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.contains(&"ide".to_string()));
        assert_eq!(
            detection.facets_patch.get("ide_id").unwrap(),
            &json!("custom-editor")
        );
        assert_eq!(detection.confidence, HIGH);
    }

    #[test]
    fn respects_override_disable_ide() {
        let detector = DeclarativeIdeDetector::new();
        let snapshot =
            create_env_snapshot(vec![("ENVSENSE_IDE", "none"), ("TERM_PROGRAM", "vscode")]);

        let detection = detector.detect(&snapshot);

        // Should not detect as IDE despite TERM_PROGRAM being present
        assert!(!detection.contexts_add.contains(&"ide".to_string()));
        assert!(detection.facets_patch.get("ide_id").is_none());
    }

    #[test]
    fn respects_override_assume_terminal() {
        let detector = DeclarativeIdeDetector::new();
        let snapshot = create_env_snapshot(vec![
            ("ENVSENSE_ASSUME_TERMINAL", "1"),
            ("TERM_PROGRAM", "vscode"),
        ]);

        let detection = detector.detect(&snapshot);

        // Should not detect as IDE despite TERM_PROGRAM being present
        assert!(!detection.contexts_add.contains(&"ide".to_string()));
        assert!(detection.facets_patch.get("ide_id").is_none());
    }
}
