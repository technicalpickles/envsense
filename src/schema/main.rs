// Main schema structure - updated in task 1.3 to use the new nested structure
use crate::detectors::DeclarativeAgentDetector;
use crate::detectors::DeclarativeCiDetector;
use crate::detectors::DeclarativeIdeDetector;
use crate::detectors::terminal::TerminalDetector;
use crate::engine::DetectionEngine;
use crate::traits::NestedTraits;
use envsense_macros::{Detection, DetectionMerger, DetectionMergerDerive};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Evidence, SCHEMA_VERSION};

/// Main schema structure using the new nested structure
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, DetectionMergerDerive)]
pub struct EnvSense {
    pub contexts: Vec<String>, // Simplified from Contexts struct
    pub traits: NestedTraits,  // New nested structure
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    pub version: String,
}

fn detect_environment() -> EnvSense {
    let engine = DetectionEngine::new()
        .register(TerminalDetector::new())
        .register(DeclarativeAgentDetector::new())
        .register(DeclarativeCiDetector::new())
        .register(DeclarativeIdeDetector::new());

    engine.detect()
}

impl EnvSense {
    pub fn detect() -> Self {
        detect_environment()
    }
}

impl Default for EnvSense {
    fn default() -> Self {
        Self {
            contexts: Vec::new(),
            traits: NestedTraits::default(),
            evidence: Vec::new(),
            version: SCHEMA_VERSION.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_serializes_with_version() {
        let envsense = EnvSense::default();
        let json = serde_json::to_string(&envsense).unwrap();
        assert!(json.contains("\"version\":\"0.3.0\""));
    }

    #[test]
    fn json_schema_generates() {
        let schema = schemars::schema_for!(EnvSense);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("EnvSense"));
    }

    #[test]
    fn new_schema_serialization() {
        let new_env = EnvSense::default();
        let json = serde_json::to_string(&new_env).unwrap();
        assert!(json.contains("\"version\":\"0.3.0\""));
        assert!(json.contains("\"traits\":"));
        assert!(json.contains("\"contexts\":"));
    }

    #[test]
    fn context_list_handling() {
        let mut new_env = EnvSense::default();
        new_env.contexts.push("agent".to_string());
        new_env.contexts.push("ide".to_string());

        let json = serde_json::to_string(&new_env).unwrap();
        assert!(json.contains("\"agent\""));
        assert!(json.contains("\"ide\""));
    }

    #[test]
    fn nested_traits_structure_validation() {
        let env = EnvSense::default();

        // Verify nested structure exists
        assert_eq!(env.traits.agent.id, None);
        assert_eq!(env.traits.ide.id, None);
        assert_eq!(env.traits.ci.id, None);
        assert!(!env.traits.terminal.interactive);

        // Verify JSON structure
        let json = serde_json::to_string_pretty(&env).unwrap();
        assert!(json.contains("\"traits\": {"));
        assert!(json.contains("\"agent\": {"));
        assert!(json.contains("\"ide\": {"));
        assert!(json.contains("\"terminal\": {"));
        assert!(json.contains("\"ci\": {"));
    }
}
