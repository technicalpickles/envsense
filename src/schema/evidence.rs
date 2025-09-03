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

    // Helper methods for common evidence patterns with nested field paths

    /// Create evidence for agent detection
    ///
    /// Creates environment variable evidence that supports agent.id field.
    /// Confidence: HIGH (1.0) - Direct env var match
    pub fn agent_detection(env_var: impl Into<String>, value: impl Into<String>) -> Self {
        Self::env_var(env_var, value).with_supports(vec!["agent.id".into()])
    }

    /// Create evidence for agent detection with both agent and host support
    ///
    /// Creates environment variable evidence that supports both agent.id and host fields.
    /// Used for environments like Replit that provide both agent and hosting context.
    /// Confidence: HIGH (1.0) - Direct env var match
    pub fn agent_with_host_detection(env_var: impl Into<String>, value: impl Into<String>) -> Self {
        Self::env_var(env_var, value).with_supports(vec!["agent.id".into(), "host".into()])
    }

    /// Create evidence for IDE detection
    ///
    /// Creates environment variable evidence that supports ide.id field.
    /// Confidence: HIGH (1.0) - Direct env var match
    pub fn ide_detection(env_var: impl Into<String>, value: impl Into<String>) -> Self {
        Self::env_var(env_var, value).with_supports(vec!["ide.id".into()])
    }

    /// Create evidence for CI detection
    ///
    /// Creates environment variable evidence that supports ci.id field.
    /// Confidence: HIGH (1.0) - Direct env var match
    pub fn ci_detection(env_var: impl Into<String>, value: impl Into<String>) -> Self {
        Self::env_var(env_var, value).with_supports(vec!["ci.id".into()])
    }

    /// Create evidence for CI detection with multiple fields
    ///
    /// Creates environment variable evidence that supports multiple CI fields.
    /// Used for CI environments that provide evidence for vendor, name, PR status, etc.
    /// Confidence: HIGH (1.0) - Direct env var match
    pub fn ci_multi_field_detection(
        env_var: impl Into<String>,
        value: impl Into<String>,
        fields: Vec<&str>,
    ) -> Self {
        let supports = fields
            .into_iter()
            .map(|field| format!("ci.{}", field))
            .collect();
        Self::env_var(env_var, value).with_supports(supports)
    }

    /// Create evidence for terminal stream TTY detection
    ///
    /// Creates TTY evidence for a specific terminal stream (stdin, stdout, stderr).
    /// Confidence: TERMINAL (1.0) - Always reliable
    pub fn terminal_stream_tty(stream: &str, is_tty: bool) -> Self {
        let field_path = format!("terminal.{}.tty", stream);
        Self::tty_trait(&field_path, is_tty).with_supports(vec![field_path])
    }

    /// Create evidence for terminal interactive detection
    ///
    /// Creates TTY evidence for terminal interactive capability.
    /// Confidence: TERMINAL (1.0) - Always reliable
    pub fn terminal_interactive(is_interactive: bool) -> Self {
        Self::tty_trait("terminal.interactive", is_interactive)
            .with_supports(vec!["terminal.interactive".into()])
    }

    /// Create evidence for terminal color level detection
    ///
    /// Creates TTY evidence for terminal color support level.
    /// Confidence: TERMINAL (1.0) - Always reliable
    pub fn terminal_color_level(color_level: impl Into<String>) -> Self {
        Self {
            signal: Signal::Tty,
            key: "terminal.color_level".into(),
            value: Some(color_level.into()),
            supports: vec!["terminal.color_level".into()],
            confidence: TERMINAL,
        }
    }

    /// Create evidence for terminal hyperlinks support
    ///
    /// Creates TTY evidence for terminal hyperlinks capability.
    /// Confidence: TERMINAL (1.0) - Always reliable
    pub fn terminal_hyperlinks(supports_hyperlinks: bool) -> Self {
        Self::tty_trait("terminal.supports_hyperlinks", supports_hyperlinks)
            .with_supports(vec!["terminal.supports_hyperlinks".into()])
    }
}
