use crate::detectors::declarative::DeclarativeDetector;
use crate::detectors::env_mapping::get_ci_mappings;
use crate::detectors::utils::SelectionStrategy;
use crate::detectors::{Detection, Detector, EnvSnapshot};
use serde_json::json;

pub struct DeclarativeCiDetector;

impl DeclarativeCiDetector {
    pub fn new() -> Self {
        Self
    }
}

impl DeclarativeDetector for DeclarativeCiDetector {
    fn get_mappings() -> Vec<crate::detectors::env_mapping::EnvMapping> {
        get_ci_mappings()
    }

    fn get_detector_type() -> &'static str {
        "ci"
    }

    fn get_context_name() -> &'static str {
        "ci"
    }

    fn get_facet_key() -> &'static str {
        "ci_id"
    }

    fn should_generate_evidence() -> bool {
        false // CI detector doesn't generate evidence for compatibility
    }

    fn get_selection_strategy() -> SelectionStrategy {
        SelectionStrategy::Confidence
    }
}

impl DeclarativeCiDetector {
    // Imperative value extraction methods removed - using declarative value mappings instead
}

impl Detector for DeclarativeCiDetector {
    fn name(&self) -> &'static str {
        "ci-declarative"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = self.create_detection(snap);

        if let Some(id) = detection
            .facets_patch
            .get("ci_id")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
        {
            // Set is_ci trait to true
            detection
                .traits_patch
                .insert("is_ci".to_string(), json!(true));

            // Create CI facet with is_ci: true
            let mut ci_facet = serde_json::Map::new();
            ci_facet.insert("is_ci".to_string(), json!(true));
            ci_facet.insert("vendor".to_string(), json!(id));

            // Add CI name based on vendor
            let ci_name = match id.as_str() {
                "github_actions" => "GitHub Actions",
                "gitlab_ci" => "GitLab CI",
                "circleci" => "CircleCI",
                "buildkite" => "Buildkite",
                "jenkins" => "Jenkins",
                "teamcity" => "TeamCity",
                "bitbucket_pipelines" => "Bitbucket Pipelines",
                "azure_pipelines" => "Azure Pipelines",
                "google_cloud_build" => "Google Cloud Build",
                "vercel" => "Vercel",
                "aws_codebuild" => "AWS CodeBuild",
                "sourcehut" => "SourceHut",
                "appveyor" => "AppVeyor",
                _ => "Generic CI",
            };
            ci_facet.insert("name".to_string(), json!(ci_name));

            // Add CI traits
            detection
                .traits_patch
                .insert("ci_vendor".to_string(), json!(id));
            detection
                .traits_patch
                .insert("ci_name".to_string(), json!(ci_name));

            // Process declarative value mappings
            let mappings = Self::get_mappings();
            for mapping in &mappings {
                if mapping.matches(&snap.env_vars) {
                    let extracted_values = mapping.extract_values(&snap.env_vars);
                    for (key, value) in extracted_values {
                        detection.traits_patch.insert(key, value);
                    }
                    break; // Use the first matching mapping
                }
            }

            detection
                .facets_patch
                .insert("ci".to_string(), json!(ci_facet));
        }

        detection
    }
}

impl Default for DeclarativeCiDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detectors::confidence::HIGH;
    use std::collections::HashMap;

    fn create_env_snapshot(env_vars: Vec<(&str, &str)>) -> EnvSnapshot {
        let mut env_map = HashMap::new();
        for (k, v) in env_vars {
            env_map.insert(k.to_string(), v.to_string());
        }

        EnvSnapshot::with_mock_tty(env_map, false, false, false)
    }

    #[test]
    fn detects_github_actions() {
        let detector = DeclarativeCiDetector::new();
        let snapshot = create_env_snapshot(vec![("GITHUB_ACTIONS", "true")]);

        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ci"]);
        assert_eq!(
            detection.facets_patch.get("ci_id").unwrap(),
            &json!("github_actions")
        );
        assert_eq!(detection.confidence, HIGH);
    }

    #[test]
    fn detects_gitlab_ci() {
        let detector = DeclarativeCiDetector::new();
        let snapshot = create_env_snapshot(vec![("GITLAB_CI", "true")]);

        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ci"]);
        assert_eq!(
            detection.facets_patch.get("ci_id").unwrap(),
            &json!("gitlab_ci")
        );
        assert_eq!(detection.confidence, HIGH);
    }

    #[test]
    fn detects_circleci() {
        let detector = DeclarativeCiDetector::new();
        let snapshot = create_env_snapshot(vec![("CIRCLECI", "true")]);

        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ci"]);
        assert_eq!(
            detection.facets_patch.get("ci_id").unwrap(),
            &json!("circleci")
        );
        assert_eq!(detection.confidence, HIGH);
    }

    #[test]
    fn detects_jenkins() {
        let detector = DeclarativeCiDetector::new();
        let snapshot = create_env_snapshot(vec![("JENKINS_URL", "http://jenkins.example.com")]);

        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ci"]);
        assert_eq!(
            detection.facets_patch.get("ci_id").unwrap(),
            &json!("jenkins")
        );
        assert_eq!(detection.confidence, HIGH);
    }

    #[test]
    fn detects_pr_status() {
        let detector = DeclarativeCiDetector::new();
        let snapshot = create_env_snapshot(vec![
            ("GITHUB_ACTIONS", "true"),
            ("GITHUB_EVENT_NAME", "pull_request"),
        ]);

        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ci"]);
        assert_eq!(detection.traits_patch.get("is_pr").unwrap(), &json!(true));
    }

    #[test]
    fn detects_branch() {
        let detector = DeclarativeCiDetector::new();
        let snapshot = create_env_snapshot(vec![
            ("GITHUB_ACTIONS", "true"),
            ("GITHUB_REF_NAME", "main"),
        ]);

        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ci"]);
        assert_eq!(
            detection.traits_patch.get("branch").unwrap(),
            &json!("main")
        );
    }

    #[test]
    fn no_detection_without_ci() {
        let detector = DeclarativeCiDetector::new();
        let snapshot = create_env_snapshot(vec![]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.is_empty());
        assert!(detection.facets_patch.is_empty());
        assert_eq!(detection.confidence, 0.0);
    }

    #[test]
    fn respects_override_force_ci() {
        let detector = DeclarativeCiDetector::new();
        let snapshot = create_env_snapshot(vec![("ENVSENSE_CI", "custom-ci")]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.contains(&"ci".to_string()));
        assert_eq!(
            detection.facets_patch.get("ci_id").unwrap(),
            &json!("custom-ci")
        );
        assert_eq!(detection.confidence, HIGH);
    }

    #[test]
    fn respects_override_disable_ci() {
        let detector = DeclarativeCiDetector::new();
        let snapshot =
            create_env_snapshot(vec![("ENVSENSE_CI", "none"), ("GITHUB_ACTIONS", "true")]);

        let detection = detector.detect(&snapshot);

        // Should not detect as CI despite GITHUB_ACTIONS being present
        assert!(!detection.contexts_add.contains(&"ci".to_string()));
        assert!(detection.facets_patch.get("ci_id").is_none());
    }

    #[test]
    fn respects_override_assume_local() {
        let detector = DeclarativeCiDetector::new();
        let snapshot = create_env_snapshot(vec![
            ("ENVSENSE_ASSUME_LOCAL", "1"),
            ("GITHUB_ACTIONS", "true"),
        ]);

        let detection = detector.detect(&snapshot);

        // Should not detect as CI despite GITHUB_ACTIONS being present
        assert!(!detection.contexts_add.contains(&"ci".to_string()));
        assert!(detection.facets_patch.get("ci_id").is_none());
    }
}
