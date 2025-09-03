use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Traits specific to CI environment detection
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Default)]
pub struct CiTraits {
    /// The detected CI system ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The CI vendor (e.g., "github", "gitlab", "jenkins")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// The CI system name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether this is a pull request build
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pr: Option<bool>,
    /// The current branch name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ci_traits() {
        let traits = CiTraits::default();
        assert_eq!(traits.id, None);
        assert_eq!(traits.vendor, None);
        assert_eq!(traits.name, None);
        assert_eq!(traits.is_pr, None);
        assert_eq!(traits.branch, None);
    }

    #[test]
    fn ci_traits_with_values() {
        let traits = CiTraits {
            id: Some("github".to_string()),
            vendor: Some("github".to_string()),
            name: Some("GitHub Actions".to_string()),
            is_pr: Some(true),
            branch: Some("main".to_string()),
        };
        assert_eq!(traits.id, Some("github".to_string()));
        assert_eq!(traits.vendor, Some("github".to_string()));
        assert_eq!(traits.name, Some("GitHub Actions".to_string()));
        assert_eq!(traits.is_pr, Some(true));
        assert_eq!(traits.branch, Some("main".to_string()));
    }

    #[test]
    fn ci_traits_serialization() {
        let traits = CiTraits {
            id: Some("gitlab".to_string()),
            vendor: Some("gitlab".to_string()),
            name: Some("GitLab CI".to_string()),
            is_pr: Some(false),
            branch: Some("feature".to_string()),
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("\"id\":\"gitlab\""));
        assert!(json.contains("\"vendor\":\"gitlab\""));
        assert!(json.contains("\"name\":\"GitLab CI\""));
        assert!(json.contains("\"is_pr\":false"));
        assert!(json.contains("\"branch\":\"feature\""));
    }

    #[test]
    fn ci_traits_deserialization() {
        let json = r#"{"id":"jenkins","vendor":"jenkins","name":"Jenkins","is_pr":false,"branch":"develop"}"#;
        let traits: CiTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.id, Some("jenkins".to_string()));
        assert_eq!(traits.vendor, Some("jenkins".to_string()));
        assert_eq!(traits.name, Some("Jenkins".to_string()));
        assert_eq!(traits.is_pr, Some(false));
        assert_eq!(traits.branch, Some("develop".to_string()));
    }

    #[test]
    fn ci_traits_partial_serialization() {
        let traits = CiTraits {
            id: Some("circleci".to_string()),
            vendor: None,
            name: None,
            is_pr: None,
            branch: None,
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("\"id\":\"circleci\""));
        assert!(!json.contains("\"vendor\""));
        assert!(!json.contains("\"name\""));
        assert!(!json.contains("\"is_pr\""));
        assert!(!json.contains("\"branch\""));
    }

    #[test]
    fn ci_traits_mixed_boolean_values() {
        let traits = CiTraits {
            id: Some("github".to_string()),
            vendor: Some("github".to_string()),
            name: Some("GitHub Actions".to_string()),
            is_pr: Some(true),
            branch: Some("feature/PR-123".to_string()),
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("\"is_pr\":true"));
        assert!(json.contains("feature/PR-123"));
    }

    #[test]
    fn ci_traits_special_characters_in_names() {
        let traits = CiTraits {
            id: Some("jenkins".to_string()),
            vendor: Some("jenkins".to_string()),
            name: Some("Jenkins Pipeline (v2.0)".to_string()),
            is_pr: Some(false),
            branch: Some("feature/🚀-rocket".to_string()),
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("Jenkins Pipeline (v2.0)"));
        assert!(json.contains("feature/🚀-rocket"));
    }

    #[test]
    fn ci_traits_deserialization_missing_fields() {
        let json = r#"{"id":"gitlab"}"#;
        let traits: CiTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.id, Some("gitlab".to_string()));
        assert_eq!(traits.vendor, None);
        assert_eq!(traits.name, None);
        assert_eq!(traits.is_pr, None);
        assert_eq!(traits.branch, None);
    }

    #[test]
    fn ci_traits_deserialization_extra_fields() {
        let json = r#"{"id":"circleci","vendor":"circleci","unknown_field":"ignored"}"#;
        let traits: CiTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.id, Some("circleci".to_string()));
        assert_eq!(traits.vendor, Some("circleci".to_string()));
        // Unknown fields should be ignored
    }

    #[test]
    fn ci_traits_edge_case_values() {
        let traits = CiTraits {
            id: Some("".to_string()), // Empty string
            vendor: Some("".to_string()),
            name: Some("".to_string()),
            is_pr: Some(false),
            branch: Some("".to_string()),
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("\"id\":\"\""));
        assert!(json.contains("\"vendor\":\"\""));
        assert!(json.contains("\"name\":\"\""));
        assert!(json.contains("\"branch\":\"\""));
    }
}
