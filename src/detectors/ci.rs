use crate::ci::{CiFacet, ci_traits, normalize_vendor};
use crate::detectors::{Detection, Detector, EnvSnapshot};
use ci_info::types::Vendor;
use serde_json::json;

pub struct CiDetector;

impl CiDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect CI environment from environment variables in the snapshot
    fn detect_ci_from_snapshot(&self, snap: &EnvSnapshot) -> CiFacet {
        // Check for various CI environment variables
        let is_ci = self.is_ci_environment(snap);

        if !is_ci {
            return CiFacet::default();
        }

        // Detect specific CI vendor
        let vendor = self.detect_vendor(snap);
        let (vendor_id, vendor_name) = vendor
            .map(normalize_vendor)
            .unwrap_or_else(|| ("generic".into(), "Generic CI".into()));

        CiFacet {
            is_ci: true,
            vendor: Some(vendor_id),
            name: Some(vendor_name),
            pr: self.detect_pr(snap).or(Some(false)), // Default to false if not detected
            branch: self.detect_branch(snap),
        }
    }

    fn is_ci_environment(&self, snap: &EnvSnapshot) -> bool {
        // Check common CI environment variables
        snap.get_env("CI")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
            || snap
                .get_env("CONTINUOUS_INTEGRATION")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false)
            || self.detect_vendor(snap).is_some()
    }

    fn detect_vendor(&self, snap: &EnvSnapshot) -> Option<Vendor> {
        // Check for specific CI vendors in order of specificity
        if snap.get_env("GITHUB_ACTIONS").is_some() {
            Some(Vendor::GitHubActions)
        } else if snap.get_env("GITLAB_CI").is_some() {
            Some(Vendor::GitLabCI)
        } else if snap.get_env("CIRCLECI").is_some() {
            Some(Vendor::CircleCI)
        } else if snap.get_env("BUILDKITE").is_some() {
            Some(Vendor::Buildkite)
        } else if snap.get_env("JENKINS_URL").is_some() || snap.get_env("JENKINS_HOME").is_some() {
            Some(Vendor::Jenkins)
        } else if snap.get_env("TEAMCITY_VERSION").is_some() {
            Some(Vendor::TeamCity)
        } else if snap.get_env("BITBUCKET_BUILD_NUMBER").is_some() {
            Some(Vendor::BitbucketPipelines)
        } else if snap.get_env("AZURE_HTTP_USER_AGENT").is_some()
            || snap.get_env("TF_BUILD").is_some()
        {
            Some(Vendor::AzurePipelines)
        } else if snap.get_env("GOOGLE_CLOUD_BUILD").is_some() {
            Some(Vendor::GoogleCloudBuild)
        } else if snap.get_env("VERCEL").is_some() {
            Some(Vendor::Vercel)
        } else if snap.get_env("CODEBUILD_BUILD_ID").is_some() {
            Some(Vendor::AWSCodeBuild)
        } else if snap.get_env("BUILD_REASON").is_some() {
            Some(Vendor::SourceHut)
        } else if snap.get_env("APPVEYOR").is_some() {
            Some(Vendor::AppVeyor)
        } else {
            None
        }
    }

    fn detect_pr(&self, snap: &EnvSnapshot) -> Option<bool> {
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

impl Detector for CiDetector {
    fn name(&self) -> &'static str {
        "ci"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        // Use snapshot-based CI detection instead of global environment
        let ci_facet = self.detect_ci_from_snapshot(snap);

        if ci_facet.is_ci {
            detection.contexts_add.push("ci".to_string());
            detection.confidence = 0.9; // High confidence for CI detection

            if let Some(vendor) = ci_facet.vendor.clone() {
                detection
                    .facets_patch
                    .insert("ci_id".to_string(), json!(vendor));
            }

            // Generate CI traits
            let traits = ci_traits(&ci_facet);
            for (key, value) in traits {
                detection.traits_patch.insert(key, value);
            }

            // Store the full CI facet data for later use
            detection
                .facets_patch
                .insert("ci".to_string(), json!(ci_facet));
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
        let detector = CiDetector::new();
        let snapshot = create_env_snapshot(vec![("GITHUB_ACTIONS", "true")]);

        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ci"]);
        assert_eq!(
            detection.facets_patch.get("ci_id").unwrap(),
            &json!("github_actions")
        );
        assert!(detection.confidence > 0.0);
    }

    #[test]
    fn detects_gitlab_ci() {
        let detector = CiDetector::new();
        let snapshot = create_env_snapshot(vec![("GITLAB_CI", "true")]);

        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ci"]);
        assert_eq!(
            detection.facets_patch.get("ci_id").unwrap(),
            &json!("gitlab_ci")
        );
        assert!(detection.confidence > 0.0);
    }

    #[test]
    fn no_detection_without_ci() {
        let detector = CiDetector::new();
        let snapshot = create_env_snapshot(vec![]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.is_empty());
        assert!(detection.facets_patch.is_empty());
        assert_eq!(detection.confidence, 0.0);
    }
}
