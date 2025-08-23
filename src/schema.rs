use crate::agent::{StdEnv, detect_agent};
use crate::envsense_ci::{CiFacet, detect_ci as detect_ci_facet};
use crate::traits::terminal::{ColorLevel, TerminalTraits};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct EnvSense {
    pub contexts: Contexts,
    pub facets: Facets,
    pub traits: Traits,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    pub version: String,
    pub rules_version: String,
}

pub const SCHEMA_VERSION: &str = "0.1.0";

impl EnvSense {
    fn detect_ide(&mut self) {
        if let Ok(term_program) = std::env::var("TERM_PROGRAM")
            && term_program == "vscode"
        {
            self.contexts.ide = true;
            self.evidence.push(Evidence {
                signal: Signal::Env,
                key: "TERM_PROGRAM".into(),
                value: Some(term_program),
                supports: vec!["ide".into()],
                confidence: 0.95,
            });

            if std::env::var("CURSOR_TRACE_ID").is_ok() {
                self.facets.ide_id = Some("cursor".into());
                self.evidence.push(Evidence {
                    signal: Signal::Env,
                    key: "CURSOR_TRACE_ID".into(),
                    value: None,
                    supports: vec!["ide_id".into()],
                    confidence: 0.95,
                });
            } else if let Ok(version) = std::env::var("TERM_PROGRAM_VERSION") {
                let ide_id = if version.to_lowercase().contains("insider") {
                    "vscode-insiders"
                } else {
                    "vscode"
                };
                self.facets.ide_id = Some(ide_id.into());
                self.evidence.push(Evidence {
                    signal: Signal::Env,
                    key: "TERM_PROGRAM_VERSION".into(),
                    value: Some(version),
                    supports: vec!["ide_id".into()],
                    confidence: 0.95,
                });
            }
        }
    }

    fn detect_agent(&mut self) {
        let det = detect_agent(&StdEnv);
        if det.agent.is_agent {
            self.contexts.agent = true;
            if let Some(id) = det.agent.name.clone() {
                self.facets.agent_id = Some(id);
            }
            if let Some(raw) = det.agent.session.get("raw").and_then(Value::as_object)
                && let Some((k, v)) = raw.iter().next()
            {
                self.evidence.push(Evidence {
                    signal: Signal::Env,
                    key: k.clone(),
                    value: v.as_str().map(|s| s.to_string()),
                    supports: vec!["agent".into(), "agent_id".into()],
                    confidence: det.agent.confidence,
                });
            }
        }
    }

    fn detect_ci(&mut self) {
        let ci = detect_ci_facet();
        if ci.is_ci {
            self.contexts.ci = true;
            if let Some(v) = ci.vendor.clone() {
                self.facets.ci_id = Some(v);
            }
        }
        self.facets.ci = ci;
    }

    fn detect_terminal(&mut self) {
        self.traits = TerminalTraits::detect().into();
    }

    pub fn detect() -> Self {
        let mut env = Self {
            contexts: Contexts::default(),
            facets: Facets::default(),
            traits: Traits::default(),
            evidence: Vec::new(),
            version: SCHEMA_VERSION.to_string(),
            rules_version: String::new(),
        };
        env.detect_terminal();
        env.detect_agent();
        env.detect_ci();
        env.detect_ide();
        env
    }
}

impl Default for EnvSense {
    fn default() -> Self {
        Self::detect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_serializes_with_version() {
        let envsense = EnvSense::default();
        let json = serde_json::to_string(&envsense).unwrap();
        assert!(json.contains("\"version\":\"0.1.0\""));
    }

    #[test]
    fn json_schema_generates() {
        let schema = schemars::schema_for!(EnvSense);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("EnvSense"));
    }
}
