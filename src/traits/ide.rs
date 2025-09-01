use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Traits specific to IDE detection
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Default)]
pub struct IdeTraits {
    /// The detected IDE ID (e.g., "cursor", "vscode", "intellij")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ide_traits() {
        let traits = IdeTraits::default();
        assert_eq!(traits.id, None);
    }

    #[test]
    fn ide_traits_with_id() {
        let traits = IdeTraits {
            id: Some("cursor".to_string()),
        };
        assert_eq!(traits.id, Some("cursor".to_string()));
    }

    #[test]
    fn ide_traits_serialization() {
        let traits = IdeTraits {
            id: Some("vscode".to_string()),
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("\"id\":\"vscode\""));
    }

    #[test]
    fn ide_traits_deserialization() {
        let json = r#"{"id":"intellij"}"#;
        let traits: IdeTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.id, Some("intellij".to_string()));
    }

    #[test]
    fn ide_traits_without_id_serialization() {
        let traits = IdeTraits { id: None };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn ide_traits_empty_string_id() {
        let traits = IdeTraits {
            id: Some("".to_string()),
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("\"id\":\"\""));
    }

    #[test]
    fn ide_traits_unicode_id() {
        let traits = IdeTraits {
            id: Some("vscode-🚀".to_string()),
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("vscode-🚀"));
    }

    #[test]
    fn ide_traits_deserialization_empty_string() {
        let json = r#"{"id":""}"#;
        let traits: IdeTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.id, Some("".to_string()));
    }

    #[test]
    fn ide_traits_deserialization_missing_field() {
        let json = r#"{}"#;
        let traits: IdeTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.id, None);
    }

    #[test]
    fn ide_traits_deserialization_extra_field() {
        let json = r#"{"id":"vscode","unknown":"ignored"}"#;
        let traits: IdeTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.id, Some("vscode".to_string()));
    }
}
