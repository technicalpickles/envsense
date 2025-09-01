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
                    traits_patch: detection.traits_patch,
                    facets_patch: detection.facets_patch, // Keep for compatibility, will be ignored
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
        result
    }
}

impl Default for DetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}
