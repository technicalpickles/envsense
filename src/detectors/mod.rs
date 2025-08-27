use std::collections::HashMap;

pub mod agent;
pub mod ci;
pub mod ide;
pub mod terminal;
pub mod tty;
pub use tty::TtyDetector;

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
    pub tty_detector: TtyDetector,
}

impl EnvSnapshot {
    /// Create snapshot with real TTY detection for production use
    /// Respects ENVSENSE_TTY_* environment variable overrides
    pub fn current() -> Self {
        let env_vars: HashMap<String, String> = std::env::vars().collect();
        
        // Check for TTY environment variable overrides
        let tty_detector = if let (Some(stdin), Some(stdout), Some(stderr)) = (
            env_vars.get("ENVSENSE_TTY_STDIN"),
            env_vars.get("ENVSENSE_TTY_STDOUT"),
            env_vars.get("ENVSENSE_TTY_STDERR"),
        ) {
            // Parse boolean values from environment variables
            let stdin_tty = stdin.parse::<bool>().unwrap_or(false);
            let stdout_tty = stdout.parse::<bool>().unwrap_or(false);
            let stderr_tty = stderr.parse::<bool>().unwrap_or(false);
            
            TtyDetector::mock(stdin_tty, stdout_tty, stderr_tty)
        } else {
            TtyDetector::real()
        };
        
        Self {
            env_vars,
            tty_detector,
        }
    }

    /// Create snapshot with mock TTY detection for testing
    pub fn for_testing(env_vars: HashMap<String, String>, tty_detector: TtyDetector) -> Self {
        Self {
            env_vars,
            tty_detector,
        }
    }

    /// Create snapshot with mock TTY detection using convenience constructor
    pub fn with_mock_tty(
        env_vars: HashMap<String, String>,
        stdin: bool,
        stdout: bool,
        stderr: bool,
    ) -> Self {
        Self {
            env_vars,
            tty_detector: TtyDetector::mock(stdin, stdout, stderr),
        }
    }

    /// Convenience methods that delegate to the TTY detector
    pub fn is_tty_stdin(&self) -> bool {
        self.tty_detector.is_tty_stdin()
    }

    pub fn is_tty_stdout(&self) -> bool {
        self.tty_detector.is_tty_stdout()
    }

    pub fn is_tty_stderr(&self) -> bool {
        self.tty_detector.is_tty_stderr()
    }

    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.env_vars.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_snapshot_with_mock_tty() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TERM".to_string(), "xterm-256color".to_string());

        let snapshot = EnvSnapshot::with_mock_tty(env_vars.clone(), true, false, false);

        assert!(snapshot.is_tty_stdin());
        assert!(!snapshot.is_tty_stdout());
        assert!(!snapshot.is_tty_stderr());
        assert_eq!(
            snapshot.get_env("TERM"),
            Some(&"xterm-256color".to_string())
        );
    }

    #[test]
    fn test_env_snapshot_for_testing() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

        let tty_detector = TtyDetector::mock_piped_io();
        let snapshot = EnvSnapshot::for_testing(env_vars.clone(), tty_detector);

        assert!(snapshot.is_tty_stdin());
        assert!(!snapshot.is_tty_stdout());
        assert!(!snapshot.is_tty_stderr());
        assert_eq!(
            snapshot.get_env("TEST_VAR"),
            Some(&"test_value".to_string())
        );
    }

    #[test]
    fn test_env_snapshot_current_uses_real_detector() {
        let snapshot = EnvSnapshot::current();

        // We can't easily test the real implementation in unit tests,
        // but we can verify it doesn't panic and returns consistent values
        let stdin1 = snapshot.is_tty_stdin();
        let stdout1 = snapshot.is_tty_stdout();
        let stderr1 = snapshot.is_tty_stderr();

        // Call again to ensure consistency
        let stdin2 = snapshot.is_tty_stdin();
        let stdout2 = snapshot.is_tty_stdout();
        let stderr2 = snapshot.is_tty_stderr();

        assert_eq!(stdin1, stdin2);
        assert_eq!(stdout1, stdout2);
        assert_eq!(stderr1, stderr2);
    }

    #[test]
    fn test_tty_detector_convenience_methods() {
        // Test all TTY
        let all_tty = TtyDetector::mock_all_tty();
        assert!(all_tty.is_tty_stdin());
        assert!(all_tty.is_tty_stdout());
        assert!(all_tty.is_tty_stderr());

        // Test no TTY
        let no_tty = TtyDetector::mock_no_tty();
        assert!(!no_tty.is_tty_stdin());
        assert!(!no_tty.is_tty_stdout());
        assert!(!no_tty.is_tty_stderr());

        // Test piped I/O
        let piped = TtyDetector::mock_piped_io();
        assert!(piped.is_tty_stdin());
        assert!(!piped.is_tty_stdout());
        assert!(!piped.is_tty_stderr());
    }
}
