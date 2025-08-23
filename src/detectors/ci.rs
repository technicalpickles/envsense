use crate::detectors::{Detector, EnvSnapshot, Detection};
use crate::ci::detect_ci as detect_ci_facet;
use serde_json::json;

pub struct CiDetector;

impl CiDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Detector for CiDetector {
    fn name(&self) -> &'static str {
        "ci"
    }
    
    fn detect(&self, _snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();
        
        // Use existing CI detection logic
        let ci_facet = detect_ci_facet();
        
        if ci_facet.is_ci {
            detection.contexts_add.push("ci".to_string());
            detection.confidence = 0.9; // High confidence for CI detection
            
            if let Some(vendor) = ci_facet.vendor.clone() {
                detection.facets_patch.insert("ci_id".to_string(), json!(vendor));
            }
            
            // Store the full CI facet data for later use
            detection.facets_patch.insert("ci".to_string(), json!(ci_facet));
        }
        
        detection
    }
}

impl Default for CiDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use temp_env::with_vars;

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
    fn detects_github_actions() {
        with_vars([("GITHUB_ACTIONS", Some("true"))], || {
            let detector = CiDetector::new();
            let snapshot = create_env_snapshot(vec![]);
            
            let detection = detector.detect(&snapshot);
            
            assert_eq!(detection.contexts_add, vec!["ci"]);
            assert_eq!(detection.facets_patch.get("ci_id").unwrap(), &json!("github_actions"));
            assert!(detection.confidence > 0.0);
        });
    }

    #[test]
    fn detects_gitlab_ci() {
        with_vars([("GITLAB_CI", Some("true"))], || {
            let detector = CiDetector::new();
            let snapshot = create_env_snapshot(vec![]);
            
            let detection = detector.detect(&snapshot);
            
            assert_eq!(detection.contexts_add, vec!["ci"]);
            assert_eq!(detection.facets_patch.get("ci_id").unwrap(), &json!("gitlab_ci"));
            assert!(detection.confidence > 0.0);
        });
    }

    #[test]
    fn no_detection_without_ci() {
        with_vars(Vec::<(&str, Option<&str>)>::new(), || {
            let detector = CiDetector::new();
            let snapshot = create_env_snapshot(vec![]);
            
            let detection = detector.detect(&snapshot);
            
            assert!(detection.contexts_add.is_empty());
            assert!(detection.facets_patch.is_empty());
            assert_eq!(detection.confidence, 0.0);
        });
    }
}