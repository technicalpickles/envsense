pub mod agent;
pub mod ci;
pub mod ide;
pub mod nested;
pub mod stream;
pub mod terminal;

pub use agent::AgentTraits;
pub use ci::CiTraits;
pub use ide::IdeTraits;
pub use nested::NestedTraits;
pub use stream::StreamInfo;
pub use terminal::{ColorLevel, TerminalTraits};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn all_traits_can_be_combined() {
        let nested = NestedTraits {
            agent: AgentTraits {
                id: Some("cursor".to_string()),
            },
            ide: IdeTraits {
                id: Some("cursor".to_string()),
            },
            terminal: TerminalTraits::detect(),
            ci: CiTraits {
                id: Some("github".to_string()),
                ..Default::default()
            },
        };

        // Test that all components work together
        assert!(nested.has_context());
        assert!(nested.is_ci());
        assert_eq!(nested.primary_agent(), Some("cursor"));
    }

    #[test]
    fn traits_serialization_roundtrip() {
        let original = NestedTraits::default();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: NestedTraits = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn traits_json_schema_integration() {
        // Test that all trait types can generate JSON schemas
        let agent_schema = schemars::schema_for!(AgentTraits);
        let ide_schema = schemars::schema_for!(IdeTraits);
        let ci_schema = schemars::schema_for!(CiTraits);
        let stream_schema = schemars::schema_for!(StreamInfo);
        let terminal_schema = schemars::schema_for!(TerminalTraits);
        let nested_schema = schemars::schema_for!(NestedTraits);

        // Ensure schemas contain expected content
        assert!(
            serde_json::to_string(&agent_schema)
                .unwrap()
                .contains("AgentTraits")
        );
        assert!(
            serde_json::to_string(&ide_schema)
                .unwrap()
                .contains("IdeTraits")
        );
        assert!(
            serde_json::to_string(&ci_schema)
                .unwrap()
                .contains("CiTraits")
        );
        assert!(
            serde_json::to_string(&stream_schema)
                .unwrap()
                .contains("StreamInfo")
        );
        assert!(
            serde_json::to_string(&terminal_schema)
                .unwrap()
                .contains("TerminalTraits")
        );
        assert!(
            serde_json::to_string(&nested_schema)
                .unwrap()
                .contains("NestedTraits")
        );
    }

    #[test]
    fn traits_edge_case_integration() {
        // Test edge cases across all trait types
        let nested = NestedTraits {
            agent: AgentTraits {
                id: Some("".to_string()),
            },
            ide: IdeTraits {
                id: Some("🚀".to_string()),
            },
            terminal: TerminalTraits {
                interactive: false,
                color_level: ColorLevel::None,
                stdin: StreamInfo {
                    tty: false,
                    piped: true,
                },
                stdout: StreamInfo {
                    tty: false,
                    piped: true,
                },
                stderr: StreamInfo {
                    tty: false,
                    piped: true,
                },
                supports_hyperlinks: false,
            },
            ci: CiTraits {
                id: Some("".to_string()),
                vendor: Some("".to_string()),
                name: Some("".to_string()),
                is_pr: Some(false),
                branch: Some("".to_string()),
            },
        };

        // Test serialization with edge cases
        let json = serde_json::to_string(&nested).unwrap();
        assert!(json.contains("\"agent\":{\"id\":\"\"}"));
        assert!(json.contains("\"ide\":{\"id\":\"🚀\"}"));
        assert!(json.contains(
            "\"ci\":{\"id\":\"\",\"vendor\":\"\",\"name\":\"\",\"is_pr\":false,\"branch\":\"\"}"
        ));

        // Test deserialization
        let deserialized: NestedTraits = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.agent.id, Some("".to_string()));
        assert_eq!(deserialized.ide.id, Some("🚀".to_string()));
        assert_eq!(deserialized.ci.id, Some("".to_string()));
    }
}
