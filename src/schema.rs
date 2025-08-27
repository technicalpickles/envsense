use crate::ci::CiFacet;
use crate::detectors::DeclarativeAgentDetector;
use crate::detectors::DeclarativeCiDetector;
use crate::detectors::DeclarativeIdeDetector;
use crate::detectors::confidence::{HIGH, MEDIUM, TERMINAL};
use crate::detectors::terminal::TerminalDetector;
use crate::engine::DetectionEngine;
use crate::traits::terminal::{ColorLevel, TerminalTraits};
use envsense_macros::{Detection, DetectionMerger, DetectionMergerDerive};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Signal {
    Env,
    Tty,
    Proc,
    Fs,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct Evidence {
    pub signal: Signal,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default)]
    pub supports: Vec<String>,
    pub confidence: f32,
}

impl Evidence {
    /// Create evidence from environment variable with value
    ///
    /// Used when we have a direct environment variable match.
    /// Confidence: HIGH (1.0) - Direct env var match
    pub fn env_var(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            signal: Signal::Env,
            key: key.into(),
            value: Some(value.into()),
            supports: Vec::new(),
            confidence: HIGH,
        }
    }

    /// Create evidence from environment variable presence
    ///
    /// Used when we know an environment variable exists but don't capture its value.
    /// Confidence: MEDIUM (0.8) - Inferred from presence
    pub fn env_presence(key: impl Into<String>) -> Self {
        Self {
            signal: Signal::Env,
            key: key.into(),
            value: None,
            supports: Vec::new(),
            confidence: MEDIUM,
        }
    }

    /// Create evidence from TTY trait detection
    ///
    /// Used for terminal capability detection which is always reliable.
    /// Confidence: TERMINAL (1.0) - Always reliable
    pub fn tty_trait(key: impl Into<String>, is_tty: bool) -> Self {
        Self {
            signal: Signal::Tty,
            key: key.into(),
            value: Some(is_tty.to_string()),
            supports: Vec::new(),
            confidence: TERMINAL,
        }
    }

    /// Add support contexts to evidence
    pub fn with_supports(mut self, supports: Vec<String>) -> Self {
        self.supports = supports;
        self
    }

    /// Override confidence level
    ///
    /// Use this sparingly - prefer the default confidence levels
    /// from the evidence constructors.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}

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
    #[serde(default)]
    pub ci: CiFacet,
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
        }
    }
}

impl From<TerminalTraits> for Traits {
    fn from(t: TerminalTraits) -> Self {
        Self {
            is_interactive: t.is_interactive,
            is_tty_stdin: t.is_tty_stdin,
            is_tty_stdout: t.is_tty_stdout,
            is_tty_stderr: t.is_tty_stderr,
            is_piped_stdin: !t.is_tty_stdin,
            is_piped_stdout: !t.is_tty_stdout,
            color_level: t.color_level,
            supports_hyperlinks: t.supports_hyperlinks,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, DetectionMergerDerive)]
pub struct EnvSense {
    pub contexts: Contexts,
    pub facets: Facets,
    pub traits: Traits,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    pub version: String,
}

pub const SCHEMA_VERSION: &str = "0.2.0";

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
}

impl Default for EnvSense {
    fn default() -> Self {
        Self {
            contexts: Contexts::default(),
            facets: Facets::default(),
            traits: Traits::default(),
            evidence: Vec::new(),
            version: SCHEMA_VERSION.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_serializes_with_version() {
        let envsense = EnvSense::default();
        let json = serde_json::to_string(&envsense).unwrap();
        assert!(json.contains("\"version\":\"0.2.0\""));
    }

    #[test]
    fn json_schema_generates() {
        let schema = schemars::schema_for!(EnvSense);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("EnvSense"));
    }
}
