use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

// Import confidence constants
use crate::detectors::confidence::{HIGH, LOW, MEDIUM};

pub trait EnvReader {
    fn get(&self, key: &str) -> Option<String>;
    fn iter(&self) -> Box<dyn Iterator<Item = (String, String)> + '_>;
}

pub struct StdEnv;

impl EnvReader for StdEnv {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
    fn iter(&self) -> Box<dyn Iterator<Item = (String, String)> + '_> {
        Box::new(std::env::vars())
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct AgentInfo {
    pub is_agent: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    pub confidence: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub session: Value,
    #[serde(default)]
    pub model: Value,
}

impl Default for AgentInfo {
    fn default() -> Self {
        Self {
            is_agent: false,
            name: None,
            vendor: None,
            variant: None,
            confidence: 0.0,
            capabilities: Vec::new(),
            session: json!({"id": null, "source": "env", "raw": {}}),
            model: json!({}),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Default)]
pub struct ContextFacets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    pub editor_confidence: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    pub host_confidence: f32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Default)]
pub struct AgentDetection {
    pub agent: AgentInfo,
    pub facets: ContextFacets,
}

fn descriptor(name: &str) -> (Option<&'static str>, Option<&'static str>, Vec<String>) {
    match name {
        "cursor" => (
            Some("Cursor"),
            Some("terminal"),
            vec!["code-edit", "run-commands", "file-ops", "tests"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        "cline" => (
            Some("Cline"),
            Some("terminal"),
            vec!["code-edit", "run-commands", "file-ops"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        "claude-code" => (
            Some("Claude Code"),
            Some("terminal"),
            vec!["code-edit", "run-commands", "file-ops"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        "amp" => (
            Some("Amp"),
            Some("terminal"),
            vec!["code-edit", "run-commands", "file-ops"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        "aider" => (
            Some("Aider"),
            Some("terminal"),
            vec!["code-edit", "run-commands", "file-ops", "git"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        "replit-agent" => (
            Some("Replit"),
            Some("terminal"),
            vec!["run-commands", "file-ops", "web-preview"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        "openhands" => (
            Some("OpenHands"),
            Some("sandbox"),
            vec!["run-commands", "file-ops", "container", "network"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        _ => (None, None, Vec::new()),
    }
}

fn is_secret(key: &str) -> bool {
    let key_upper = key.to_uppercase();
    key_upper.ends_with("_KEY")
        || key_upper.ends_with("_TOKEN")
        || key_upper.ends_with("_SECRET")
        || key_upper.contains("API_KEY")
}

fn add_raw(agent: &mut AgentInfo, key: &str, value: &str) {
    if is_secret(key) {
        return;
    }
    if let Some(obj) = agent.session.get_mut("raw").and_then(Value::as_object_mut) {
        obj.insert(key.to_string(), Value::String(value.to_string()));
    }
}

fn detect_editor(vars: &HashMap<String, String>, facets: &mut ContextFacets) {
    if vars
        .get("TERM_PROGRAM")
        .map(|v| v == "vscode")
        .unwrap_or(false)
        || vars.keys().any(|k| k.starts_with("VSCODE_"))
    {
        facets.editor = Some("vscode".into());
        facets.editor_confidence = 0.6;
    } else if vars.keys().any(|k| k.starts_with("JETBRAINS_CLIENT_")) {
        facets.editor = Some("jetbrains".into());
        facets.editor_confidence = 0.6;
    }
}

fn detect_replit(
    vars: &HashMap<String, String>,
    detection: &mut AgentDetection,
    allow_agent: bool,
) {
    if let Some(v) = vars.get("REPL_ID") {
        if allow_agent && detection.agent.name.is_none() {
            detection.agent.name = Some("replit-agent".into());
            detection.agent.confidence = 0.9;
            detection.agent.is_agent = true;
            add_raw(&mut detection.agent, "REPL_ID", v);
        }
        detection.facets.host = Some("replit".into());
        detection.facets.host_confidence = 0.9;
    } else if vars.contains_key("REPLIT_USER")
        || vars.contains_key("REPLIT_DEV_DOMAIN")
        || vars.contains_key("REPLIT_DEPLOYMENT")
    {
        detection.facets.host = Some("replit".into());
        detection.facets.host_confidence = 0.6;
        if allow_agent
            && detection.agent.name.is_none()
            && vars.get("IS_CODE_AGENT").map(|v| v == "1").unwrap_or(false)
        {
            detection.agent.name = Some("replit-agent".into());
            detection.agent.confidence = 0.8;
            detection.agent.is_agent = true;
        }
    }
}

fn detect_host(vars: &HashMap<String, String>, facets: &mut ContextFacets) {
    if facets.host.is_some() {
        return;
    }
    if vars.contains_key("CODESPACES")
        || vars.contains_key("GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN")
    {
        facets.host = Some("codespaces".into());
        facets.host_confidence = 0.6;
    } else if vars
        .get("GITHUB_ACTIONS")
        .map(|v| v == "1")
        .unwrap_or(false)
        || vars.get("CI").map(|v| v == "1").unwrap_or(false)
    {
        facets.host = Some("ci".into());
        facets.host_confidence = 0.6;
    } else {
        facets.host = Some("unknown".into());
        facets.host_confidence = 0.5;
    }
}

pub fn detect_agent(env: &impl EnvReader) -> AgentDetection {
    let mut detection = AgentDetection::default();
    let vars: HashMap<String, String> = env.iter().collect();

    // overrides
    if vars
        .get("ENVSENSE_ASSUME_HUMAN")
        .map(|v| v == "1")
        .unwrap_or(false)
        || vars
            .get("ENVSENSE_AGENT")
            .map(|v| v == "none")
            .unwrap_or(false)
    {
        detect_editor(&vars, &mut detection.facets);
        detect_replit(&vars, &mut detection, false);
        detect_host(&vars, &mut detection.facets);
        return detection;
    }

    if let Some(slug) = vars.get("ENVSENSE_AGENT") {
        detection.agent.is_agent = true;
        detection.agent.name = Some(slug.clone());
        let (vendor, variant, caps) = descriptor(slug);
        detection.agent.vendor = vendor.map(str::to_string);
        detection.agent.variant = variant.map(str::to_string);
        detection.agent.capabilities = caps;
        detection.agent.confidence = HIGH; // Direct override
    }

    detect_editor(&vars, &mut detection.facets);
    detect_replit(&vars, &mut detection, true);
    detect_host(&vars, &mut detection.facets);

    if detection.agent.name.is_none() {
        if let Some(v) = vars.get("CURSOR_AGENT") {
            detection.agent.name = Some("cursor".into());
            detection.agent.confidence = HIGH; // Direct env var
            detection.agent.is_agent = true;
            add_raw(&mut detection.agent, "CURSOR_AGENT", v);
        } else if let Some(v) = vars.get("CLINE_ACTIVE") {
            detection.agent.name = Some("cline".into());
            detection.agent.confidence = HIGH; // Direct env var
            detection.agent.is_agent = true;
            add_raw(&mut detection.agent, "CLINE_ACTIVE", v);
        } else if let Some(v) = vars.get("CLAUDECODE") {
            detection.agent.name = Some("claude-code".into());
            detection.agent.confidence = HIGH; // Direct env var
            detection.agent.is_agent = true;
            add_raw(&mut detection.agent, "CLAUDECODE", v);
        } else if vars.keys().any(|k| k.starts_with("SANDBOX_")) {
            detection.agent.name = Some("openhands".into());
            detection.agent.confidence = MEDIUM; // Inferred from context
            detection.agent.is_agent = true;
        } else if vars.get("IS_CODE_AGENT").map(|v| v == "1").unwrap_or(false) {
            detection.agent.name = Some("unknown".into());
            detection.agent.confidence = LOW; // Heuristic
            detection.agent.is_agent = true;
        }
    }

    if detection.agent.name.is_none() {
        // aider weak signals
        let aider_envs: Vec<&String> = vars.keys().filter(|k| k.starts_with("AIDER_")).collect();
        let aider_detect = vars.contains_key("AIDER_MODEL") || aider_envs.len() >= 2;
        if aider_detect {
            detection.agent.name = Some("aider".into());
            detection.agent.confidence = MEDIUM; // Inferred from context
            detection.agent.is_agent = true;
        }
    }

    if let Some(name) = detection.agent.name.clone()
        && (detection.agent.vendor.is_none()
            || detection.agent.variant.is_none()
            || detection.agent.capabilities.is_empty())
    {
        let (vendor, variant, caps) = descriptor(&name);
        if detection.agent.vendor.is_none() {
            detection.agent.vendor = vendor.map(str::to_string);
        }
        if detection.agent.variant.is_none() {
            detection.agent.variant = variant.map(str::to_string);
        }
        if detection.agent.capabilities.is_empty() {
            detection.agent.capabilities = caps;
        }
    }

    if let Some(m) = vars.get("AIDER_MODEL") {
        detection.agent.model = json!({"name": m, "source": "env"});
    } else if let Some(m) = vars.get("ANTHROPIC_MODEL") {
        detection.agent.model = json!({"name": m, "provider": "anthropic", "source": "env"});
    } else if let Some(m) = vars.get("OPENAI_MODEL") {
        detection.agent.model = json!({"name": m, "provider": "openai", "source": "env"});
    }

    detection
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestEnv<'a> {
        vars: HashMap<&'a str, &'a str>,
    }

    impl<'a> EnvReader for TestEnv<'a> {
        fn get(&self, key: &str) -> Option<String> {
            self.vars.get(key).map(|v| v.to_string())
        }
        fn iter(&self) -> Box<dyn Iterator<Item = (String, String)> + '_> {
            Box::new(
                self.vars
                    .iter()
                    .map(|(k, v)| ((*k).to_string(), (*v).to_string())),
            )
        }
    }

    #[test]
    fn table_driven_detection() {
        struct Case {
            name: &'static str,
            env: Vec<(&'static str, &'static str)>,
            expected_agent: Option<&'static str>,
            expected_is_agent: bool,
            expected_host: Option<&'static str>,
        }
        let cases = vec![
            Case {
                name: "cursor_terminal",
                env: vec![("CURSOR_AGENT", "1"), ("TERM_PROGRAM", "vscode")],
                expected_agent: Some("cursor"),
                expected_is_agent: true,
                expected_host: None,
            },
            Case {
                name: "cline_basic",
                env: vec![("CLINE_ACTIVE", "true")],
                expected_agent: Some("cline"),
                expected_is_agent: true,
                expected_host: None,
            },
            Case {
                name: "claude_code",
                env: vec![("CLAUDECODE", "1")],
                expected_agent: Some("claude-code"),
                expected_is_agent: true,
                expected_host: None,
            },
            Case {
                name: "replit_full",
                env: vec![("REPL_ID", "abc"), ("REPLIT_USER", "josh")],
                expected_agent: Some("replit-agent"),
                expected_is_agent: true,
                expected_host: Some("replit"),
            },
            Case {
                name: "replit_weak",
                env: vec![("REPLIT_USER", "josh")],
                expected_agent: None,
                expected_is_agent: false,
                expected_host: Some("replit"),
            },
            Case {
                name: "openhands",
                env: vec![
                    ("SANDBOX_VOLUMES", "..."),
                    ("SANDBOX_RUNTIME_CONTAINER_IMAGE", "..."),
                ],
                expected_agent: Some("openhands"),
                expected_is_agent: true,
                expected_host: None,
            },
            Case {
                name: "aider",
                env: vec![("AIDER_MODEL", "gpt-4o-mini")],
                expected_agent: Some("aider"),
                expected_is_agent: true,
                expected_host: None,
            },
            Case {
                name: "vscode_only",
                env: vec![("TERM_PROGRAM", "vscode")],
                expected_agent: None,
                expected_is_agent: false,
                expected_host: Some("unknown"),
            },
            Case {
                name: "override_force_human",
                env: vec![("ENVSENSE_ASSUME_HUMAN", "1"), ("CURSOR_AGENT", "1")],
                expected_agent: None,
                expected_is_agent: false,
                expected_host: Some("unknown"),
            },
            Case {
                name: "override_force_agent",
                env: vec![("ENVSENSE_AGENT", "cursor")],
                expected_agent: Some("cursor"),
                expected_is_agent: true,
                expected_host: Some("unknown"),
            },
        ];

        for case in cases {
            let map: HashMap<&str, &str> = case.env.into_iter().collect();
            let env = TestEnv { vars: map };
            let det = detect_agent(&env);
            assert_eq!(
                det.agent.name.as_deref(),
                case.expected_agent,
                "{}",
                case.name
            );
            assert_eq!(det.agent.is_agent, case.expected_is_agent, "{}", case.name);
            if let Some(h) = case.expected_host {
                assert_eq!(det.facets.host.as_deref(), Some(h), "{}", case.name);
            }
        }
    }
}
