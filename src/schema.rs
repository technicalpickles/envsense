use crate::ci::{CiFacet, detect_ci as detect_ci_facet};
use crate::detectors::{Detector, EnvSnapshot};
use crate::detectors::agent::AgentDetector;
use crate::detectors::ci::CiDetector;
use crate::detectors::ide::IdeDetector;
use crate::traits::terminal::{ColorLevel, TerminalTraits};
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

fn detect_ide(env: &mut EnvSense) {
    let detector = IdeDetector::new();
    let snapshot = EnvSnapshot::current();
    let detection = detector.detect(&snapshot);
    
    for context in detection.contexts_add {
        if context == "ide" {
            env.contexts.ide = true;
        }
    }
    
    if let Some(ide_id_value) = detection.facets_patch.get("ide_id") {
        if let Some(ide_id) = ide_id_value.as_str() {
            env.facets.ide_id = Some(ide_id.to_string());
        }
    }
    
    env.evidence.extend(detection.evidence);
}

fn detect_agent_env(env: &mut EnvSense) {
    let detector = AgentDetector::new();
    let snapshot = EnvSnapshot::current();
    let detection = detector.detect(&snapshot);
    
    for context in detection.contexts_add {
        if context == "agent" {
            env.contexts.agent = true;
        }
    }
    
    if let Some(agent_id_value) = detection.facets_patch.get("agent_id") {
        if let Some(agent_id) = agent_id_value.as_str() {
            env.facets.agent_id = Some(agent_id.to_string());
        }
    }
    
    env.evidence.extend(detection.evidence);
}

fn detect_ci_env(env: &mut EnvSense) {
    let detector = CiDetector::new();
    let snapshot = EnvSnapshot::current();
    let detection = detector.detect(&snapshot);
    
    for context in detection.contexts_add {
        if context == "ci" {
            env.contexts.ci = true;
        }
    }
    
    if let Some(ci_id_value) = detection.facets_patch.get("ci_id") {
        if let Some(ci_id) = ci_id_value.as_str() {
            env.facets.ci_id = Some(ci_id.to_string());
        }
    }
    
    // Extract the CI facet from detection if present
    if let Some(ci_facet_value) = detection.facets_patch.get("ci") {
        if let Ok(ci_facet) = serde_json::from_value::<CiFacet>(ci_facet_value.clone()) {
            env.facets.ci = ci_facet;
        }
    } else {
        // Fallback to legacy detection if detector doesn't provide CI facet
        env.facets.ci = detect_ci_facet();
    }
}

fn detect_terminal_env(env: &mut EnvSense) {
    env.traits = TerminalTraits::detect().into();
}

fn detect_environment() -> EnvSense {
    let mut env = EnvSense {
        contexts: Contexts::default(),
        facets: Facets::default(),
        traits: Traits::default(),
        evidence: Vec::new(),
        version: SCHEMA_VERSION.to_string(),
        rules_version: String::new(),
    };
    detect_terminal_env(&mut env);
    detect_agent_env(&mut env);
    detect_ci_env(&mut env);
    detect_ide(&mut env);
    env
}

impl EnvSense {
    pub fn detect() -> Self {
        detect_environment()
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
