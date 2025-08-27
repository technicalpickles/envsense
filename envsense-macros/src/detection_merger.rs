//! Detection merging trait and utilities

use std::collections::HashMap;

/// Represents a single detection result from a detector
#[derive(Debug, Clone)]
pub struct Detection {
    pub contexts_add: Vec<String>,
    pub traits_patch: HashMap<String, serde_json::Value>,
    pub facets_patch: HashMap<String, serde_json::Value>,
    pub evidence: Vec<serde_json::Value>, // Generic evidence for now
    pub confidence: f32,
}

/// Trait for types that can merge multiple detection results
pub trait DetectionMerger {
    /// Merge multiple detection results into this instance
    fn merge_detections(&mut self, detections: &[Detection]);
}
