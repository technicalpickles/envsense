use crate::detectors::confidence::{HIGH, MEDIUM, TERMINAL};
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
