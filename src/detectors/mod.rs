use std::collections::HashMap;

pub mod agent;
pub mod ci;
pub mod ide;
pub mod terminal;

pub trait Detector {
    fn name(&self) -> &'static str;
    fn detect(&self, snap: &EnvSnapshot) -> Detection;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Detection {
    pub contexts_add: Vec<String>,
    pub traits_patch: HashMap<String, serde_json::Value>,
    pub facets_patch: HashMap<String, serde_json::Value>,
    pub evidence: Vec<crate::schema::Evidence>,
    pub confidence: f32,
}

impl Default for Detection {
    fn default() -> Self {
        Self {
            contexts_add: Vec::new(),
            traits_patch: HashMap::new(),
            facets_patch: HashMap::new(),
            evidence: Vec::new(),
            confidence: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvSnapshot {
    pub env_vars: HashMap<String, String>,
    pub is_tty_stdin: bool,
    pub is_tty_stdout: bool,
    pub is_tty_stderr: bool,
}

impl EnvSnapshot {
    pub fn current() -> Self {
        use std::io::IsTerminal;

        let env_vars: HashMap<String, String> = std::env::vars().collect();

        Self {
            env_vars,
            is_tty_stdin: std::io::stdin().is_terminal(),
            is_tty_stdout: std::io::stdout().is_terminal(),
            is_tty_stderr: std::io::stderr().is_terminal(),
        }
    }

    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.env_vars.get(key)
    }
}
