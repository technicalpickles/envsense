use std::collections::HashMap;

pub mod agent;
pub mod ci;
pub mod ide;
pub mod terminal;

/// Confidence levels for detection results
/// 
/// These constants provide clear, meaningful confidence values
/// with documented reasoning for when to use each level.
pub mod confidence {
    /// Direct environment variable match (e.g., TERM_PROGRAM=vscode)
    /// 
    /// Used when we have a clear, unambiguous signal from the environment.
    /// This is the highest confidence level for environment-based detection.
    /// 
    /// Examples:
    /// - `CI=true` - Direct CI environment indicator
    /// - `TERM_PROGRAM=vscode` - Direct IDE environment indicator
    /// - `CURSOR_AGENT=1` - Direct agent environment indicator
    pub const HIGH: f32 = 1.0;
    
    /// Inferred from context (e.g., multiple environment variables)
    /// 
    /// Used when we have strong but indirect evidence that requires
    /// interpretation or combination of multiple signals.
    /// 
    /// Examples:
    /// - Multiple `AIDER_*` variables indicating Aider agent
    /// - Multiple `SANDBOX_*` variables indicating sandboxed environment
    /// - Combination of SSH variables indicating remote session
    pub const MEDIUM: f32 = 0.8;
    
    /// Heuristic detection (e.g., file path patterns, process names)
    /// 
    /// Used when we have weak or circumstantial evidence that requires
    /// pattern matching or heuristic analysis.
    /// 
    /// Examples:
    /// - Process name patterns (e.g., "cursor" in process tree)
    /// - File system markers (e.g., `.vscode` directory)
    /// - Network connection patterns
    pub const LOW: f32 = 0.6;
    
    /// Terminal detection (always reliable)
    /// 
    /// Used for TTY detection which is always accurate because it
    /// directly queries the operating system for terminal capabilities.
    /// 
    /// Examples:
    /// - `isatty()` calls for stdin/stdout/stderr
    /// - Terminal capability detection
    /// - Color support detection
    pub const TERMINAL: f32 = 1.0;
}

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

        // Allow overriding TTY detection for testing
        let is_tty_stdin = env_vars
            .get("ENVSENSE_TTY_STDIN")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or_else(|| std::io::stdin().is_terminal());

        let is_tty_stdout = env_vars
            .get("ENVSENSE_TTY_STDOUT")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or_else(|| std::io::stdout().is_terminal());

        let is_tty_stderr = env_vars
            .get("ENVSENSE_TTY_STDERR")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or_else(|| std::io::stderr().is_terminal());

        Self {
            env_vars,
            is_tty_stdin,
            is_tty_stdout,
            is_tty_stderr,
        }
    }

    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.env_vars.get(key)
    }
}
