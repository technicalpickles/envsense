use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Traits specific to agent detection
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq, Default)]
pub struct AgentTraits {
    /// The detected agent ID (e.g., "cursor", "vscode", "intellij")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_agent_traits() {
        let traits = AgentTraits::default();
        assert_eq!(traits.id, None);
    }

    #[test]
    fn agent_traits_with_id() {
        let traits = AgentTraits {
            id: Some("cursor".to_string()),
        };
        assert_eq!(traits.id, Some("cursor".to_string()));
    }

    #[test]
    fn agent_traits_serialization() {
        let traits = AgentTraits {
            id: Some("vscode".to_string()),
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("\"id\":\"vscode\""));
    }

    #[test]
    fn agent_traits_deserialization() {
        let json = r#"{"id":"intellij"}"#;
        let traits: AgentTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.id, Some("intellij".to_string()));
    }

    #[test]
    fn agent_traits_without_id_serialization() {
        let traits = AgentTraits { id: None };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn agent_traits_empty_string_id() {
        let traits = AgentTraits {
            id: Some("".to_string()),
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("\"id\":\"\""));
    }

    #[test]
    fn agent_traits_unicode_id() {
        let traits = AgentTraits {
            id: Some("cursor-🚀".to_string()),
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("cursor-🚀"));
    }

    #[test]
    fn agent_traits_deserialization_empty_string() {
        let json = r#"{"id":""}"#;
        let traits: AgentTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.id, Some("".to_string()));
    }

    #[test]
    fn agent_traits_deserialization_missing_field() {
        let json = r#"{}"#;
        let traits: AgentTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.id, None);
    }

    #[test]
    fn agent_traits_deserialization_extra_field() {
        let json = r#"{"id":"cursor","unknown":"ignored"}"#;
        let traits: AgentTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.id, Some("cursor".to_string()));
    }
}
