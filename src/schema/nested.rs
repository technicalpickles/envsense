use crate::traits::NestedTraits;
use envsense_macros::{Detection, DetectionMerger, DetectionMergerDerive};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::evidence::Evidence;

/// New nested schema structure using the nested traits system
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, DetectionMergerDerive)]
pub struct NewEnvSense {
    pub contexts: Vec<String>, // Simplified from Contexts struct
    pub traits: NestedTraits,  // New nested structure
    pub evidence: Vec<Evidence>,
    pub version: String,
}

impl Default for NewEnvSense {
    fn default() -> Self {
        Self {
            contexts: Vec::new(),
            traits: NestedTraits::default(),
            evidence: Vec::new(),
            version: super::SCHEMA_VERSION.to_string(),
        }
    }
}
