// Main schema structure - this will be updated in task 1.3 to use the new nested structure
use crate::detectors::DeclarativeAgentDetector;
use crate::detectors::DeclarativeCiDetector;
use crate::detectors::DeclarativeIdeDetector;
use crate::detectors::terminal::TerminalDetector;
use crate::engine::DetectionEngine;
use envsense_macros::{Detection, DetectionMerger, DetectionMergerDerive};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Contexts, Evidence, Facets, SCHEMA_VERSION, Traits};

/// Main schema structure (keeping external interface for now)
/// This will be updated in task 1.3 to use the new nested structure
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, DetectionMergerDerive)]
pub struct EnvSense {
    pub contexts: Contexts,
    pub facets: Facets,
    pub traits: Traits,
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
            contexts: Contexts::default(),
            facets: Facets::default(),
            traits: Traits::default(),
            evidence: Vec::new(),
            version: SCHEMA_VERSION.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{LegacyEnvSense, NewEnvSense};

    #[test]
    fn default_serializes_with_version() {
        let envsense = EnvSense::default();
        let json = serde_json::to_string(&envsense).unwrap();
        assert!(json.contains("\"version\":\"0.2.0\""));
    }

    #[test]
    fn json_schema_generates() {
        let schema = schemars::schema_for!(EnvSense);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("EnvSense"));
    }

    #[test]
    fn new_schema_serialization() {
        let new_env = NewEnvSense::default();
        let json = serde_json::to_string(&new_env).unwrap();
        assert!(json.contains("\"version\":\"0.2.0\""));
        assert!(json.contains("\"traits\":"));
        assert!(json.contains("\"contexts\":"));
    }

    #[test]
    fn legacy_conversion() {
        let legacy = LegacyEnvSense::default();
        let new = NewEnvSense::from_legacy(&legacy);
        let back = new.to_legacy();
        assert_eq!(legacy, back);
    }

    #[test]
    fn context_list_handling() {
        let mut new_env = NewEnvSense::default();
        new_env.contexts.push("agent".to_string());
        new_env.contexts.push("ide".to_string());

        let json = serde_json::to_string(&new_env).unwrap();
        assert!(json.contains("\"agent\""));
        assert!(json.contains("\"ide\""));
    }
}
