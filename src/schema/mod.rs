pub mod evidence;
pub mod legacy;
pub mod main;
pub mod nested;

// Re-export commonly used types
pub use evidence::{Evidence, Signal};
pub use legacy::{Contexts, Facets, LegacyEnvSense, Traits};
pub use main::EnvSense;
pub use nested::NewEnvSense;

// Schema version constants
pub const SCHEMA_VERSION: &str = "0.2.0"; // Keep at 0.2.0 until task 1.3
pub const LEGACY_SCHEMA_VERSION: &str = "0.2.0"; // Will be different in task 1.3

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::terminal::ColorLevel;

    #[test]
    fn schema_version_constants() {
        assert_eq!(SCHEMA_VERSION, "0.2.0");
        assert_eq!(LEGACY_SCHEMA_VERSION, "0.2.0");
    }

    #[test]
    fn legacy_envsense_default() {
        let legacy = LegacyEnvSense::default();
        assert_eq!(legacy.version, LEGACY_SCHEMA_VERSION);
        assert_eq!(legacy.contexts.agent, false);
        assert_eq!(legacy.contexts.ide, false);
        assert_eq!(legacy.contexts.ci, false);
    }

    #[test]
    fn new_envsense_default() {
        let new = NewEnvSense::default();
        assert_eq!(new.version, SCHEMA_VERSION);
        assert!(new.contexts.is_empty());
        assert_eq!(new.traits.agent.id, None);
        assert_eq!(new.traits.ide.id, None);
        assert_eq!(new.traits.ci.id, None);
    }

    #[test]
    fn conversion_roundtrip() {
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
            evidence: vec![Evidence::env_var("TERM", "xterm-256color")],
            version: LEGACY_SCHEMA_VERSION.to_string(),
        };

        let new = NewEnvSense::from_legacy(&legacy);

        // Verify contexts were converted correctly
        assert!(new.contexts.contains(&"agent".to_string()));
        assert!(new.contexts.contains(&"ide".to_string()));
        assert!(!new.contexts.contains(&"ci".to_string()));

        // Verify traits were converted correctly
        assert_eq!(new.traits.agent.id, Some("cursor".to_string()));
        assert_eq!(new.traits.ide.id, Some("cursor".to_string()));
        assert!(new.traits.terminal.interactive);
        assert_eq!(new.traits.terminal.color_level, ColorLevel::Truecolor);
        assert!(new.traits.terminal.supports_hyperlinks);

        // Verify evidence was preserved
        assert_eq!(new.evidence.len(), 1);
        assert_eq!(new.evidence[0].key, "TERM");

        // Convert back and verify
        let back = new.to_legacy();
        assert_eq!(back.contexts.agent, true);
        assert_eq!(back.contexts.ide, true);
        assert_eq!(back.facets.agent_id, Some("cursor".to_string()));
        assert_eq!(back.traits.is_interactive, true);
    }

    #[test]
    fn ci_traits_conversion() {
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
            evidence: vec![Evidence::env_var("CI", "true")],
            version: LEGACY_SCHEMA_VERSION.to_string(),
        };

        let new = NewEnvSense::from_legacy(&legacy);

        // Verify CI context
        assert!(new.contexts.contains(&"ci".to_string()));

        // Verify CI traits
        assert_eq!(new.traits.ci.id, Some("github".to_string()));
        assert_eq!(new.traits.ci.vendor, Some("github".to_string()));
        assert_eq!(new.traits.ci.name, Some("GitHub Actions".to_string()));
        assert_eq!(new.traits.ci.is_pr, Some(true));
        assert_eq!(new.traits.ci.branch, Some("main".to_string()));

        // Verify terminal traits
        assert!(!new.traits.terminal.interactive);
        assert!(new.traits.terminal.stdin.piped);
        assert!(new.traits.terminal.stdout.piped);

        // Convert back and verify
        let back = new.to_legacy();
        assert_eq!(back.contexts.ci, true);
        assert_eq!(back.facets.ci_id, Some("github".to_string()));
        assert_eq!(back.traits.is_ci, Some(true));
        assert_eq!(back.traits.ci_vendor, Some("github".to_string()));
    }

    #[test]
    fn empty_contexts_handling() {
        let new = NewEnvSense::default();
        assert!(new.contexts.is_empty());

        let json = serde_json::to_string(&new).unwrap();
        assert!(json.contains("\"contexts\":[]"));
    }

    #[test]
    fn nested_traits_serialization() {
        let new = NewEnvSense::default();
        let json = serde_json::to_string(&new.traits).unwrap();
        assert!(json.contains("\"agent\":"));
        assert!(json.contains("\"terminal\":"));
        assert!(json.contains("\"ide\":"));
        assert!(json.contains("\"ci\":"));
    }
}
