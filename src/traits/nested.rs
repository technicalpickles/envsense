use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::agent::AgentTraits;
use super::ci::CiTraits;
use super::ide::IdeTraits;
use super::terminal::TerminalTraits;

/// Combined traits structure that organizes all environment traits by context
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Default)]
pub struct NestedTraits {
    /// Agent-related traits (e.g., cursor, vscode, intellij)
    pub agent: AgentTraits,
    /// IDE-related traits (e.g., cursor, vscode, intellij)
    pub ide: IdeTraits,
    /// Terminal-related traits (interactive, color support, stream info)
    pub terminal: TerminalTraits,
    /// CI environment traits (vendor, name, PR status, branch)
    pub ci: CiTraits,
}

impl NestedTraits {
    /// Create nested traits with detected values
    pub fn detect() -> Self {
        Self {
            agent: AgentTraits::default(), // Will be populated by detection engine
            ide: IdeTraits::default(),     // Will be populated by detection engine
            terminal: TerminalTraits::detect(),
            ci: CiTraits::default(), // Will be populated by detection engine
        }
    }

    /// Check if any context is detected
    pub fn has_context(&self) -> bool {
        self.agent.id.is_some() || self.ide.id.is_some() || self.ci.id.is_some()
    }

    /// Check if running in a CI environment
    pub fn is_ci(&self) -> bool {
        self.ci.id.is_some()
    }

    /// Check if running in an interactive terminal
    pub fn is_interactive(&self) -> bool {
        self.terminal.interactive
    }

    /// Get the primary agent ID (agent takes precedence over IDE)
    pub fn primary_agent(&self) -> Option<&str> {
        self.agent.id.as_deref().or(self.ide.id.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::super::stream::StreamInfo;
    use super::super::terminal::ColorLevel;
    use super::*;

    #[test]
    fn default_nested_traits() {
        let traits = NestedTraits::default();
        assert_eq!(traits.agent.id, None);
        assert_eq!(traits.ide.id, None);
        assert!(!traits.terminal.interactive);
        assert_eq!(traits.ci.id, None);
    }

    #[test]
    fn nested_traits_serialization() {
        let traits = NestedTraits {
            agent: AgentTraits {
                id: Some("cursor".to_string()),
            },
            ide: IdeTraits {
                id: Some("cursor".to_string()),
            },
            terminal: TerminalTraits {
                interactive: true,
                color_level: ColorLevel::Truecolor,
                stdin: StreamInfo {
                    tty: true,
                    piped: false,
                },
                stdout: StreamInfo {
                    tty: true,
                    piped: false,
                },
                stderr: StreamInfo {
                    tty: true,
                    piped: false,
                },
                supports_hyperlinks: true,
            },
            ci: CiTraits {
                id: Some("github".to_string()),
                vendor: Some("github".to_string()),
                name: Some("GitHub Actions".to_string()),
                is_pr: Some(true),
                branch: Some("main".to_string()),
            },
        };

        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("\"agent\":{\"id\":\"cursor\"}"));
        assert!(json.contains("\"ide\":{\"id\":\"cursor\"}"));
        assert!(json.contains("\"terminal\":{\"interactive\":true"));
        assert!(json.contains("\"ci\":{\"id\":\"github\",\"vendor\":\"github\",\"name\":\"GitHub Actions\",\"is_pr\":true,\"branch\":\"main\"}"));
    }

    #[test]
    fn nested_traits_deserialization() {
        let json = r#"{
            "agent": {"id": "vscode"},
            "ide": {"id": "vscode"},
            "terminal": {
                "interactive": false,
                "color_level": "none",
                "stdin": {"tty": false, "piped": true},
                "stdout": {"tty": false, "piped": true},
                "stderr": {"tty": false, "piped": true},
                "supports_hyperlinks": false
            },
            "ci": {"id": "gitlab", "vendor": "gitlab", "name": "GitLab CI", "is_pr": false, "branch": "develop"}
        }"#;

        let traits: NestedTraits = serde_json::from_str(json).unwrap();
        assert_eq!(traits.agent.id, Some("vscode".to_string()));
        assert_eq!(traits.ide.id, Some("vscode".to_string()));
        assert!(!traits.terminal.interactive);
        assert_eq!(traits.ci.id, Some("gitlab".to_string()));
        assert_eq!(traits.ci.vendor, Some("gitlab".to_string()));
    }

    #[test]
    fn context_detection() {
        let mut traits = NestedTraits::default();
        assert!(!traits.has_context());

        traits.agent.id = Some("cursor".to_string());
        assert!(traits.has_context());

        traits.agent.id = None;
        assert!(!traits.has_context());

        traits.ci.id = Some("github".to_string());
        assert!(traits.has_context());
    }

    #[test]
    fn ci_detection() {
        let mut traits = NestedTraits::default();
        assert!(!traits.is_ci());

        traits.ci.id = Some("github".to_string());
        assert!(traits.is_ci());
    }

    #[test]
    fn interactive_detection() {
        let mut traits = NestedTraits::default();
        assert!(!traits.is_interactive());

        traits.terminal.interactive = true;
        assert!(traits.is_interactive());
    }

    #[test]
    fn primary_agent_precedence() {
        let mut traits = NestedTraits::default();
        assert_eq!(traits.primary_agent(), None);

        traits.ide.id = Some("vscode".to_string());
        assert_eq!(traits.primary_agent(), Some("vscode"));

        traits.agent.id = Some("cursor".to_string());
        assert_eq!(traits.primary_agent(), Some("cursor")); // Agent takes precedence
    }

    #[test]
    fn nested_traits_partial_context_detection() {
        let mut traits = NestedTraits::default();

        // Test each context individually
        traits.agent.id = Some("cursor".to_string());
        assert!(traits.has_context());
        assert_eq!(traits.primary_agent(), Some("cursor"));

        traits.agent.id = None;
        traits.ide.id = Some("vscode".to_string());
        assert!(traits.has_context());
        assert_eq!(traits.primary_agent(), Some("vscode"));

        traits.ide.id = None;
        traits.ci.id = Some("github".to_string());
        assert!(traits.has_context());
        assert_eq!(traits.primary_agent(), None); // CI doesn't count as primary agent
    }

    #[test]
    fn nested_traits_multiple_contexts() {
        let mut traits = NestedTraits::default();

        // Set multiple contexts
        traits.agent.id = Some("cursor".to_string());
        traits.ide.id = Some("vscode".to_string());
        traits.ci.id = Some("github".to_string());

        assert!(traits.has_context());
        assert!(traits.is_ci());
        assert_eq!(traits.primary_agent(), Some("cursor")); // Agent takes precedence
    }

    #[test]
    fn nested_traits_detect_method() {
        let traits = NestedTraits::detect();

        // Verify that detection runs without panicking and returns a valid structure
        // The actual values depend on the environment (CI vs local vs Docker)
        // so we just verify the structure is populated and internally consistent

        // Interactive should be consistent with TTY detection
        let expected_interactive = traits.terminal.stdin.tty && traits.terminal.stdout.tty;
        assert_eq!(
            traits.terminal.interactive, expected_interactive,
            "Interactive field should match stdin.tty && stdout.tty logic"
        );

        // TTY and piped should be opposites for each stream
        assert_eq!(
            traits.terminal.stdin.tty, !traits.terminal.stdin.piped,
            "stdin.tty and stdin.piped should be opposites"
        );
        assert_eq!(
            traits.terminal.stdout.tty, !traits.terminal.stdout.piped,
            "stdout.tty and stdout.piped should be opposites"
        );
        assert_eq!(
            traits.terminal.stderr.tty, !traits.terminal.stderr.piped,
            "stderr.tty and stderr.piped should be opposites"
        );

        // Others should be default (not populated by detection engine yet)
        assert_eq!(traits.agent.id, None);
        assert_eq!(traits.ide.id, None);
        assert_eq!(traits.ci.id, None);
    }

    #[test]
    fn nested_traits_json_schema_generation() {
        // Ensure JSON schema can be generated
        let schema = schemars::schema_for!(NestedTraits);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("NestedTraits"));
        assert!(json.contains("agent"));
        assert!(json.contains("ide"));
        assert!(json.contains("terminal"));
        assert!(json.contains("ci"));
    }

    #[test]
    fn nested_traits_serialization_roundtrip() {
        let original = NestedTraits::default();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: NestedTraits = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn nested_traits_edge_case_serialization() {
        let traits = NestedTraits {
            agent: AgentTraits {
                id: Some("".to_string()),
            },
            ide: IdeTraits {
                id: Some("".to_string()),
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

        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("\"agent\":{\"id\":\"\"}"));
        assert!(json.contains("\"ide\":{\"id\":\"\"}"));
        assert!(json.contains(
            "\"ci\":{\"id\":\"\",\"vendor\":\"\",\"name\":\"\",\"is_pr\":false,\"branch\":\"\"}"
        ));
    }
}
