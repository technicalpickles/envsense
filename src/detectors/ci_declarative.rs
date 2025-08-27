use crate::detectors::env_mapping::get_ci_mappings;
use crate::detectors::{Detection, Detector, EnvSnapshot};
use crate::schema::Evidence;
use serde_json::json;

pub struct DeclarativeCiDetector;

impl DeclarativeCiDetector {
    pub fn new() -> Self {
        Self
    }

    fn detect_ci(&self, snap: &EnvSnapshot) -> (Option<String>, f32, Vec<Evidence>) {
        let mappings = get_ci_mappings();
        let mut ci_id = None;
        let mut confidence = 0.0;
        let evidence = Vec::new();

        // Find the highest confidence matching CI
        for mapping in &mappings {
            if mapping.matches(&snap.env_vars) && mapping.confidence > confidence {
                ci_id = mapping.facets.get("ci_id").cloned();
                confidence = mapping.confidence;
                break; // Take the first (highest confidence) match
            }
        }

        (ci_id, confidence, evidence)
    }

    fn detect_pr_status(&self, snap: &EnvSnapshot) -> Option<bool> {
        // GitHub Actions
        if let Some(event_name) = snap.get_env("GITHUB_EVENT_NAME") {
            return Some(event_name == "pull_request");
        }

        // GitLab CI
        if let Some(merge_request_id) = snap.get_env("CI_MERGE_REQUEST_ID") {
            return Some(!merge_request_id.is_empty());
        }

        // CircleCI
        if let Some(pr_number) = snap.get_env("CIRCLE_PR_NUMBER") {
            return Some(!pr_number.is_empty());
        }

        // Generic CI_PULL_REQUEST
        if let Some(pr) = snap.get_env("CI_PULL_REQUEST") {
            return Some(pr.to_lowercase() == "true" || pr == "1");
        }

        None
    }

    fn detect_branch(&self, snap: &EnvSnapshot) -> Option<String> {
        // Try various branch environment variables
        snap.get_env("GITHUB_REF_NAME")
            .cloned()
            .or_else(|| snap.get_env("CI_COMMIT_REF_NAME").cloned())
            .or_else(|| snap.get_env("CIRCLE_BRANCH").cloned())
            .or_else(|| snap.get_env("BRANCH_NAME").cloned())
            .or_else(|| snap.get_env("GIT_BRANCH").cloned())
    }
}

impl Detector for DeclarativeCiDetector {
    fn name(&self) -> &'static str {
        "ci-declarative"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        let (ci_id, confidence, evidence) = self.detect_ci(snap);

        if let Some(id) = ci_id {
            detection.contexts_add.push("ci".to_string());
            detection
                .facets_patch
                .insert("ci_id".to_string(), json!(id));
            detection.confidence = confidence;

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

            // Add PR status if available
            if let Some(is_pr) = self.detect_pr_status(snap) {
                detection
                    .traits_patch
                    .insert("is_pr".to_string(), json!(is_pr));
                detection
                    .traits_patch
                    .insert("ci_pr".to_string(), json!(is_pr));
                ci_facet.insert("pr".to_string(), json!(is_pr));
            } else {
                // Default to false if not detected
                detection
                    .traits_patch
                    .insert("ci_pr".to_string(), json!(false));
                ci_facet.insert("pr".to_string(), json!(false));
            }

            // Add branch name if available
            if let Some(branch) = self.detect_branch(snap) {
                detection
                    .traits_patch
                    .insert("branch".to_string(), json!(branch));
                ci_facet.insert("branch".to_string(), json!(branch));
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
}
