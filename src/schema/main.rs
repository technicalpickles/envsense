// Main schema structure - updated in task 1.3 to use the new nested structure
use crate::detectors::DeclarativeAgentDetector;
use crate::detectors::DeclarativeCiDetector;
use crate::detectors::DeclarativeIdeDetector;
use crate::detectors::terminal::TerminalDetector;
use crate::engine::DetectionEngine;
use crate::traits::NestedTraits;
use envsense_macros::{Detection, DetectionMerger, DetectionMergerDerive};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Evidence, LegacyEnvSense, SCHEMA_VERSION};

/// Main schema structure using the new nested structure
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, DetectionMergerDerive)]
pub struct EnvSense {
    pub contexts: Vec<String>, // Simplified from Contexts struct
    pub traits: NestedTraits,  // New nested structure
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    pub version: String,
}

fn detect_environment() -> EnvSense {
    let engine = DetectionEngine::new()
        .register(TerminalDetector::new())
        .register(DeclarativeAgentDetector::new())
        .register(DeclarativeCiDetector::new())
        .register(DeclarativeIdeDetector::new());

    engine.detect()
}

impl EnvSense {
    pub fn detect() -> Self {
        detect_environment()
    }

    /// Convert new schema to legacy format for backward compatibility
    pub fn to_legacy(&self) -> LegacyEnvSense {
        use super::LEGACY_SCHEMA_VERSION;
        use super::legacy::{Contexts, Facets, Traits};

        LegacyEnvSense {
            contexts: Contexts {
                agent: self.contexts.contains(&"agent".to_string()),
                ide: self.contexts.contains(&"ide".to_string()),
                ci: self.contexts.contains(&"ci".to_string()),
                container: self.contexts.contains(&"container".to_string()),
                remote: self.contexts.contains(&"remote".to_string()),
            },
            facets: Facets {
                agent_id: self.traits.agent.id.clone(),
                ide_id: self.traits.ide.id.clone(),
                ci_id: self.traits.ci.id.clone(),
                container_id: None, // New schema doesn't have container info yet
                host: None,         // Host concept removed
            },
            traits: Traits {
                is_interactive: self.traits.terminal.interactive,
                is_tty_stdin: self.traits.terminal.stdin.tty,
                is_tty_stdout: self.traits.terminal.stdout.tty,
                is_tty_stderr: self.traits.terminal.stderr.tty,
                is_piped_stdin: self.traits.terminal.stdin.piped,
                is_piped_stdout: self.traits.terminal.stdout.piped,
                color_level: self.traits.terminal.color_level.clone(),
                supports_hyperlinks: self.traits.terminal.supports_hyperlinks,
                is_ci: if self.traits.ci.id.is_some() {
                    Some(true)
                } else {
                    None
                },
                ci_vendor: self.traits.ci.vendor.clone(),
                ci_name: self.traits.ci.name.clone(),
                is_pr: self.traits.ci.is_pr,
                ci_pr: self.traits.ci.is_pr,
                branch: self.traits.ci.branch.clone(),
            },
            evidence: self.evidence.clone(),
            version: LEGACY_SCHEMA_VERSION.to_string(),
        }
    }

    /// Convert legacy schema to new format
    pub fn from_legacy(legacy: &LegacyEnvSense) -> Self {
        use crate::traits::{AgentTraits, CiTraits, IdeTraits, StreamInfo, TerminalTraits};

        let mut contexts = Vec::new();
        if legacy.contexts.agent {
            contexts.push("agent".to_string());
        }
        if legacy.contexts.ide {
            contexts.push("ide".to_string());
        }
        if legacy.contexts.ci {
            contexts.push("ci".to_string());
        }
        if legacy.contexts.container {
            contexts.push("container".to_string());
        }
        if legacy.contexts.remote {
            contexts.push("remote".to_string());
        }

        Self {
            contexts,
            traits: NestedTraits {
                agent: AgentTraits {
                    id: legacy.facets.agent_id.clone(),
                },
                ide: IdeTraits {
                    id: legacy.facets.ide_id.clone(),
                },
                terminal: TerminalTraits {
                    interactive: legacy.traits.is_interactive,
                    color_level: legacy.traits.color_level.clone(),
                    stdin: StreamInfo {
                        tty: legacy.traits.is_tty_stdin,
                        piped: legacy.traits.is_piped_stdin,
                    },
                    stdout: StreamInfo {
                        tty: legacy.traits.is_tty_stdout,
                        piped: legacy.traits.is_piped_stdout,
                    },
                    stderr: StreamInfo {
                        tty: legacy.traits.is_tty_stderr,
                        piped: false, // Legacy doesn't have stderr piped info
                    },
                    supports_hyperlinks: legacy.traits.supports_hyperlinks,
                },
                ci: CiTraits {
                    id: legacy.facets.ci_id.clone(),
                    vendor: legacy.traits.ci_vendor.clone(),
                    name: legacy.traits.ci_name.clone(),
                    is_pr: legacy.traits.is_pr.or(legacy.traits.ci_pr),
                    branch: legacy.traits.branch.clone(),
                },
            },
            // Host concept removed from new schema
            evidence: legacy.evidence.clone(),
            version: SCHEMA_VERSION.to_string(),
        }
    }
}

impl Default for EnvSense {
    fn default() -> Self {
        Self {
            contexts: Vec::new(),
            traits: NestedTraits::default(),
            evidence: Vec::new(),
            version: SCHEMA_VERSION.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::LegacyEnvSense;

    #[test]
    fn default_serializes_with_version() {
        let envsense = EnvSense::default();
        let json = serde_json::to_string(&envsense).unwrap();
        assert!(json.contains("\"version\":\"0.3.0\""));
    }

    #[test]
    fn json_schema_generates() {
        let schema = schemars::schema_for!(EnvSense);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("EnvSense"));
    }

    #[test]
    fn new_schema_serialization() {
        let new_env = EnvSense::default();
        let json = serde_json::to_string(&new_env).unwrap();
        assert!(json.contains("\"version\":\"0.3.0\""));
        assert!(json.contains("\"traits\":"));
        assert!(json.contains("\"contexts\":"));
    }

    #[test]
    fn legacy_conversion_roundtrip() {
        let legacy = LegacyEnvSense::default();
        let new = EnvSense::from_legacy(&legacy);
        let back = new.to_legacy();
        assert_eq!(legacy, back);
    }

    #[test]
    fn legacy_conversion_with_data() {
        use crate::schema::legacy::{Contexts, Facets, Traits};
        use crate::traits::terminal::ColorLevel;

        // Create legacy schema with comprehensive test data
        let legacy = LegacyEnvSense {
            contexts: Contexts {
                agent: true,
                ide: true,
                ci: false,
                container: false,
                remote: false,
            },
            facets: Facets {
                agent_id: Some("cursor".to_string()),
                ide_id: Some("cursor".to_string()),
                ci_id: None,
                container_id: None,
                host: None,
            },
            traits: Traits {
                is_interactive: true,
                is_tty_stdin: true,
                is_tty_stdout: true,
                is_tty_stderr: true,
                is_piped_stdin: false,
                is_piped_stdout: false,
                color_level: ColorLevel::Truecolor,
                supports_hyperlinks: true,
                is_ci: Some(false),
                ci_vendor: None,
                ci_name: None,
                is_pr: None,
                ci_pr: None,
                branch: None,
            },
            evidence: vec![],
            version: crate::schema::LEGACY_SCHEMA_VERSION.to_string(),
        };

        // Convert to new format
        let new = EnvSense::from_legacy(&legacy);

        // Verify contexts conversion
        assert!(new.contexts.contains(&"agent".to_string()));
        assert!(new.contexts.contains(&"ide".to_string()));
        assert!(!new.contexts.contains(&"ci".to_string()));
        assert_eq!(new.contexts.len(), 2);

        // Verify traits conversion
        assert_eq!(new.traits.agent.id, Some("cursor".to_string()));
        assert_eq!(new.traits.ide.id, Some("cursor".to_string()));
        assert!(new.traits.terminal.interactive);
        assert_eq!(new.traits.terminal.color_level, ColorLevel::Truecolor);
        assert!(new.traits.terminal.supports_hyperlinks);
        assert!(new.traits.terminal.stdin.tty);
        assert!(new.traits.terminal.stdout.tty);
        assert!(new.traits.terminal.stderr.tty);
        assert!(!new.traits.terminal.stdin.piped);
        assert!(!new.traits.terminal.stdout.piped);
        assert_eq!(new.traits.ci.id, None);
        assert_eq!(new.version, "0.3.0");

        // Convert back to legacy
        let back = new.to_legacy();
        assert_eq!(back.contexts.agent, true);
        assert_eq!(back.contexts.ide, true);
        assert_eq!(back.contexts.ci, false);
        assert_eq!(back.facets.agent_id, Some("cursor".to_string()));
        assert_eq!(back.facets.ide_id, Some("cursor".to_string()));
        assert_eq!(back.traits.is_interactive, true);
        assert_eq!(back.traits.color_level, ColorLevel::Truecolor);
        assert_eq!(back.version, "0.2.0");
    }

    #[test]
    fn ci_environment_conversion() {
        use crate::schema::legacy::{Contexts, Facets, Traits};
        use crate::traits::terminal::ColorLevel;

        // Test CI environment conversion
        let legacy = LegacyEnvSense {
            contexts: Contexts {
                agent: false,
                ide: false,
                ci: true,
                container: false,
                remote: false,
            },
            facets: Facets {
                agent_id: None,
                ide_id: None,
                ci_id: Some("github".to_string()),
                container_id: None,
                host: None,
            },
            traits: Traits {
                is_interactive: false,
                is_tty_stdin: false,
                is_tty_stdout: false,
                is_tty_stderr: false,
                is_piped_stdin: true,
                is_piped_stdout: true,
                color_level: ColorLevel::None,
                supports_hyperlinks: false,
                is_ci: Some(true),
                ci_vendor: Some("github".to_string()),
                ci_name: Some("GitHub Actions".to_string()),
                is_pr: Some(true),
                ci_pr: Some(true),
                branch: Some("main".to_string()),
            },
            evidence: vec![],
            version: crate::schema::LEGACY_SCHEMA_VERSION.to_string(),
        };

        let new = EnvSense::from_legacy(&legacy);

        // Verify CI context
        assert!(new.contexts.contains(&"ci".to_string()));
        assert!(!new.contexts.contains(&"agent".to_string()));
        assert!(!new.contexts.contains(&"ide".to_string()));

        // Verify CI traits
        assert_eq!(new.traits.ci.id, Some("github".to_string()));
        assert_eq!(new.traits.ci.vendor, Some("github".to_string()));
        assert_eq!(new.traits.ci.name, Some("GitHub Actions".to_string()));
        assert_eq!(new.traits.ci.is_pr, Some(true));
        assert_eq!(new.traits.ci.branch, Some("main".to_string()));

        // Verify terminal traits for CI
        assert!(!new.traits.terminal.interactive);
        assert!(new.traits.terminal.stdin.piped);
        assert!(new.traits.terminal.stdout.piped);
        assert_eq!(new.traits.terminal.color_level, ColorLevel::None);

        // Convert back and verify
        let back = new.to_legacy();
        assert_eq!(back.contexts.ci, true);
        assert_eq!(back.facets.ci_id, Some("github".to_string()));
        assert_eq!(back.traits.is_ci, Some(true));
        assert_eq!(back.traits.ci_vendor, Some("github".to_string()));
        assert_eq!(back.traits.ci_name, Some("GitHub Actions".to_string()));
        assert_eq!(back.traits.is_pr, Some(true));
        assert_eq!(back.traits.branch, Some("main".to_string()));
    }

    #[test]
    fn version_management() {
        // Test that version constants are correct
        assert_eq!(super::SCHEMA_VERSION, "0.3.0");
        assert_eq!(crate::schema::LEGACY_SCHEMA_VERSION, "0.2.0");

        // Test that new schema uses correct version
        let new = EnvSense::default();
        assert_eq!(new.version, "0.3.0");

        // Test that legacy schema uses correct version
        let legacy = LegacyEnvSense::default();
        assert_eq!(legacy.version, "0.2.0");

        // Test that conversions maintain correct versions
        let converted_to_new = EnvSense::from_legacy(&legacy);
        assert_eq!(converted_to_new.version, "0.3.0");

        let converted_to_legacy = converted_to_new.to_legacy();
        assert_eq!(converted_to_legacy.version, "0.2.0");
    }

    #[test]
    fn context_list_handling() {
        let mut new_env = EnvSense::default();
        new_env.contexts.push("agent".to_string());
        new_env.contexts.push("ide".to_string());

        let json = serde_json::to_string(&new_env).unwrap();
        assert!(json.contains("\"agent\""));
        assert!(json.contains("\"ide\""));
    }

    #[test]
    fn nested_traits_structure_validation() {
        let env = EnvSense::default();

        // Verify nested structure exists
        assert_eq!(env.traits.agent.id, None);
        assert_eq!(env.traits.ide.id, None);
        assert_eq!(env.traits.ci.id, None);
        assert!(!env.traits.terminal.interactive);

        // Verify JSON structure
        let json = serde_json::to_string_pretty(&env).unwrap();
        assert!(json.contains("\"traits\": {"));
        assert!(json.contains("\"agent\": {"));
        assert!(json.contains("\"ide\": {"));
        assert!(json.contains("\"terminal\": {"));
        assert!(json.contains("\"ci\": {"));
    }

    #[test]
    fn edge_case_conversions() {
        use crate::schema::legacy::{Contexts, Facets, Traits};
        use crate::traits::terminal::ColorLevel;

        // Test conversion with all contexts enabled
        let legacy = LegacyEnvSense {
            contexts: Contexts {
                agent: true,
                ide: true,
                ci: true,
                container: true,
                remote: true,
            },
            facets: Facets {
                agent_id: Some("".to_string()), // Empty string
                ide_id: Some("vscode".to_string()),
                ci_id: Some("gitlab".to_string()),
                container_id: Some("docker".to_string()),
                host: Some("localhost".to_string()),
            },
            traits: Traits {
                is_interactive: false,
                is_tty_stdin: false,
                is_tty_stdout: true,
                is_tty_stderr: false,
                is_piped_stdin: true,
                is_piped_stdout: false,
                color_level: ColorLevel::Ansi16,
                supports_hyperlinks: false,
                is_ci: Some(true),
                ci_vendor: Some("gitlab".to_string()),
                ci_name: Some("GitLab CI".to_string()),
                is_pr: Some(false),
                ci_pr: Some(false),
                branch: Some("develop".to_string()),
            },
            evidence: vec![],
            version: crate::schema::LEGACY_SCHEMA_VERSION.to_string(),
        };

        let new = EnvSense::from_legacy(&legacy);

        // Verify all contexts are preserved
        assert_eq!(new.contexts.len(), 5);
        assert!(new.contexts.contains(&"agent".to_string()));
        assert!(new.contexts.contains(&"ide".to_string()));
        assert!(new.contexts.contains(&"ci".to_string()));
        assert!(new.contexts.contains(&"container".to_string()));
        assert!(new.contexts.contains(&"remote".to_string()));

        // Verify empty string is preserved
        assert_eq!(new.traits.agent.id, Some("".to_string()));
        assert_eq!(new.traits.ide.id, Some("vscode".to_string()));
        assert_eq!(new.traits.ci.id, Some("gitlab".to_string()));

        // Verify mixed stream states
        assert!(!new.traits.terminal.stdin.tty);
        assert!(new.traits.terminal.stdin.piped);
        assert!(new.traits.terminal.stdout.tty);
        assert!(!new.traits.terminal.stdout.piped);
        assert!(!new.traits.terminal.stderr.tty);

        // Convert back and verify preservation
        let back = new.to_legacy();
        assert_eq!(back.facets.agent_id, Some("".to_string()));
        assert_eq!(back.facets.ide_id, Some("vscode".to_string()));
        assert_eq!(back.facets.ci_id, Some("gitlab".to_string()));
        assert_eq!(back.traits.ci_vendor, Some("gitlab".to_string()));
        assert_eq!(back.traits.is_pr, Some(false));
    }

    #[test]
    fn json_schema_generation() {
        // Verify that JSON schema can be generated for the new structure
        let schema = schemars::schema_for!(EnvSense);
        let json = serde_json::to_string(&schema).unwrap();

        assert!(json.contains("EnvSense"));
        assert!(json.contains("contexts"));
        assert!(json.contains("traits"));
        assert!(json.contains("evidence"));
        assert!(json.contains("version"));

        // Verify nested traits are in schema
        assert!(
            json.contains("NestedTraits") || json.contains("agent") && json.contains("terminal")
        );
    }

    #[test]
    fn serialization_deserialization_roundtrip() {
        let mut env = EnvSense::default();
        env.contexts.push("agent".to_string());
        env.contexts.push("ci".to_string());
        env.traits.agent.id = Some("cursor".to_string());
        env.traits.ci.vendor = Some("github".to_string());
        env.traits.terminal.interactive = true;

        // Serialize to JSON
        let json = serde_json::to_string(&env).unwrap();

        // Deserialize back
        let deserialized: EnvSense = serde_json::from_str(&json).unwrap();

        // Verify roundtrip preservation
        assert_eq!(env, deserialized);
        assert_eq!(deserialized.contexts.len(), 2);
        assert!(deserialized.contexts.contains(&"agent".to_string()));
        assert!(deserialized.contexts.contains(&"ci".to_string()));
        assert_eq!(deserialized.traits.agent.id, Some("cursor".to_string()));
        assert_eq!(deserialized.traits.ci.vendor, Some("github".to_string()));
        assert!(deserialized.traits.terminal.interactive);
    }
}
