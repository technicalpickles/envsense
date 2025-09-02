use crate::detectors::{Detector, EnvSnapshot};
use crate::schema::{EnvSense, SCHEMA_VERSION};
use crate::traits::NestedTraits;
use envsense_macros::DetectionMerger;

pub struct DetectionEngine {
    detectors: Vec<Box<dyn Detector>>,
}

impl DetectionEngine {
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    pub fn register<D: Detector + 'static>(mut self, detector: D) -> Self {
        self.detectors.push(Box::new(detector));
        self
    }

    pub fn detect(&self) -> EnvSense {
        let snapshot = EnvSnapshot::current();
        self.detect_from_snapshot(&snapshot)
    }

    pub fn detect_from_snapshot(&self, snapshot: &EnvSnapshot) -> EnvSense {
        let mut result = EnvSense {
            contexts: Vec::new(),
            traits: NestedTraits::default(),
            evidence: Vec::new(),
            version: SCHEMA_VERSION.to_string(),
        };

        // Collect all detections
        let detections: Vec<envsense_macros::Detection> = self
            .detectors
            .iter()
            .map(|detector| {
                let detection = detector.detect(snapshot);
                envsense_macros::Detection {
                    contexts_add: detection.contexts_add,
                    traits_patch: detection.traits_patch, // Now contains nested objects
                    facets_patch: detection.facets_patch, // Legacy support
                    evidence: detection
                        .evidence
                        .into_iter()
                        .map(|e| serde_json::to_value(e).unwrap())
                        .collect(),
                    confidence: detection.confidence,
                }
            })
            .collect();

        // Use the macro-generated merging logic
        result.merge_detections(&detections);

        // Validate the nested structure (development aid)
        if cfg!(debug_assertions)
            && let Err(validation_error) = self.validate_nested_structure(&result)
        {
            eprintln!(
                "DEBUG: Nested structure validation warning: {}",
                validation_error
            );
        }

        result
    }

    /// Validate the nested structure for debugging during development
    fn validate_nested_structure(&self, result: &EnvSense) -> Result<(), String> {
        // Validate that nested traits are properly structured

        // Check that agent context matches agent traits
        let has_agent_context = result.contexts.contains(&"agent".to_string());
        let has_agent_id = result.traits.agent.id.is_some();
        if has_agent_context && !has_agent_id {
            return Err("Agent context present but agent.id is None".to_string());
        }

        // Check that IDE context matches IDE traits
        let has_ide_context = result.contexts.contains(&"ide".to_string());
        let has_ide_id = result.traits.ide.id.is_some();
        if has_ide_context && !has_ide_id {
            return Err("IDE context present but ide.id is None".to_string());
        }

        // Check that CI context matches CI traits
        let has_ci_context = result.contexts.contains(&"ci".to_string());
        let has_ci_id = result.traits.ci.id.is_some();
        if has_ci_context && !has_ci_id {
            return Err("CI context present but ci.id is None".to_string());
        }

        // Check that evidence field paths reference valid nested fields
        for evidence in &result.evidence {
            for supported_field in &evidence.supports {
                if !self.is_valid_nested_field_path(supported_field) {
                    return Err(format!(
                        "Evidence references invalid field path: {}",
                        supported_field
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check if a field path is valid for the nested structure
    fn is_valid_nested_field_path(&self, field_path: &str) -> bool {
        // Valid nested field paths for the new schema
        matches!(
            field_path,
            // Agent fields
            "agent.id" |
            // IDE fields
            "ide.id" |
            // Terminal fields
            "terminal.interactive" |
            "terminal.color_level" |
            "terminal.stdin.tty" |
            "terminal.stdin.piped" |
            "terminal.stdout.tty" |
            "terminal.stdout.piped" |
            "terminal.stderr.tty" |
            "terminal.stderr.piped" |
            "terminal.supports_hyperlinks" |
            // CI fields
            "ci.id" |
            "ci.vendor" |
            "ci.name" |
            "ci.is_pr" |
            "ci.branch" |
            // Legacy flat fields (for backward compatibility)
            "agent_id" |
            "ide_id" |
            "ci_id" |
            "is_interactive" |
            "is_tty_stdin" |
            "is_tty_stdout" |
            "is_tty_stderr" |
            "is_piped_stdin" |
            "is_piped_stdout" |
            "color_level" |
            "supports_hyperlinks" |
            "ci_vendor" |
            "ci_name" |
            "is_pr" |
            "branch" |
            // Context fields
            "agent" |
            "ide" |
            "ci" |
            "container" |
            "remote"
        )
    }
}

impl Default for DetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}
