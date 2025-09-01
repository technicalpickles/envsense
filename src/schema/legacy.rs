use crate::traits::terminal::{ColorLevel, TerminalTraits};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::evidence::Evidence;

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
pub struct Contexts {
    pub agent: bool,
    pub ide: bool,
    pub ci: bool,
    pub container: bool,
    pub remote: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
pub struct Facets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ide_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    // Legacy CiFacet removed - CI information now comes from declarative detection via traits
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct Traits {
    pub is_interactive: bool,
    pub is_tty_stdin: bool,
    pub is_tty_stdout: bool,
    pub is_tty_stderr: bool,
    pub is_piped_stdin: bool,
    pub is_piped_stdout: bool,
    pub color_level: ColorLevel,
    pub supports_hyperlinks: bool,
    // CI-related traits added by declarative CI detection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_ci: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_pr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

impl Default for Traits {
    fn default() -> Self {
        Self {
            is_interactive: false,
            is_tty_stdin: false,
            is_tty_stdout: false,
            is_tty_stderr: false,
            is_piped_stdin: false,
            is_piped_stdout: false,
            color_level: ColorLevel::None,
            supports_hyperlinks: false,
            // CI-related traits default to None
            is_ci: None,
            ci_vendor: None,
            ci_name: None,
            is_pr: None,
            ci_pr: None,
            branch: None,
        }
    }
}

impl From<TerminalTraits> for Traits {
    fn from(t: TerminalTraits) -> Self {
        Self {
            is_interactive: t.interactive,
            is_tty_stdin: t.stdin.tty,
            is_tty_stdout: t.stdout.tty,
            is_tty_stderr: t.stderr.tty,
            is_piped_stdin: t.stdin.piped,
            is_piped_stdout: t.stdout.piped,
            color_level: t.color_level,
            supports_hyperlinks: t.supports_hyperlinks,
            // CI-related traits default to None for terminal traits
            is_ci: None,
            ci_vendor: None,
            ci_name: None,
            is_pr: None,
            ci_pr: None,
            branch: None,
        }
    }
}

/// Legacy schema structure for backward compatibility
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct LegacyEnvSense {
    pub contexts: Contexts,
    pub facets: Facets,
    pub traits: Traits,
    pub evidence: Vec<Evidence>,
    pub version: String,
}

impl Default for LegacyEnvSense {
    fn default() -> Self {
        Self {
            contexts: Contexts::default(),
            facets: Facets::default(),
            traits: Traits::default(),
            evidence: Vec::new(),
            version: super::LEGACY_SCHEMA_VERSION.to_string(),
        }
    }
}
