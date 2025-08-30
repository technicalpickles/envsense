use crate::detectors::EnvSnapshot;
use std::collections::HashMap;

/// Create an environment snapshot for testing with the given environment variables
///
/// This is a shared utility function used across multiple test modules to create
/// consistent test environment snapshots with mock TTY settings.
///
/// # Arguments
///
/// * `env_vars` - Vector of (key, value) pairs representing environment variables
///
/// # Returns
///
/// An `EnvSnapshot` with the specified environment variables and mock TTY settings
/// (stdin: false, stdout: false, stderr: false)
///
/// # Example
///
/// ```rust
/// use envsense::detectors::test_utils::create_env_snapshot;
///
/// let snapshot = create_env_snapshot(vec![
///     ("GITHUB_ACTIONS", "true"),
///     ("GITHUB_REF_NAME", "main"),
/// ]);
/// ```
pub fn create_env_snapshot(env_vars: Vec<(&str, &str)>) -> EnvSnapshot {
    let mut env_map = HashMap::new();
    for (k, v) in env_vars {
        env_map.insert(k.to_string(), v.to_string());
    }

    EnvSnapshot::with_mock_tty(env_map, false, false, false)
}

/// Create an environment snapshot for testing with custom TTY settings
///
/// This function allows creating test environment snapshots with specific TTY
/// configurations, useful for testing terminal-related functionality.
///
/// # Arguments
///
/// * `env_vars` - Vector of (key, value) pairs representing environment variables
/// * `tty_stdin` - Whether stdin should be a TTY
/// * `tty_stdout` - Whether stdout should be a TTY  
/// * `tty_stderr` - Whether stderr should be a TTY
///
/// # Returns
///
/// An `EnvSnapshot` with the specified environment variables and TTY settings
///
/// # Example
///
/// ```rust
/// use envsense::detectors::test_utils::create_env_snapshot_with_tty;
///
/// let snapshot = create_env_snapshot_with_tty(
///     vec![("TERM", "xterm-256color")],
///     true,  // stdin is TTY
///     true,  // stdout is TTY
///     false, // stderr is not TTY
/// );
/// ```
pub fn create_env_snapshot_with_tty(
    env_vars: Vec<(&str, &str)>,
    tty_stdin: bool,
    tty_stdout: bool,
    tty_stderr: bool,
) -> EnvSnapshot {
    let mut env_map = HashMap::new();
    for (k, v) in env_vars {
        env_map.insert(k.to_string(), v.to_string());
    }

    EnvSnapshot::with_mock_tty(env_map, tty_stdin, tty_stdout, tty_stderr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_env_snapshot() {
        let snapshot = create_env_snapshot(vec![
            ("TEST_KEY", "test_value"),
            ("ANOTHER_KEY", "another_value"),
        ]);

        assert_eq!(
            snapshot.get_env("TEST_KEY"),
            Some(&"test_value".to_string())
        );
        assert_eq!(
            snapshot.get_env("ANOTHER_KEY"),
            Some(&"another_value".to_string())
        );
        assert_eq!(snapshot.get_env("MISSING_KEY"), None);

        // Default TTY settings should be false
        assert!(!snapshot.is_tty_stdin());
        assert!(!snapshot.is_tty_stdout());
        assert!(!snapshot.is_tty_stderr());
    }

    #[test]
    fn test_create_env_snapshot_with_tty() {
        let snapshot = create_env_snapshot_with_tty(
            vec![("TEST_KEY", "test_value")],
            true,  // stdin is TTY
            false, // stdout is not TTY
            true,  // stderr is TTY
        );

        assert_eq!(
            snapshot.get_env("TEST_KEY"),
            Some(&"test_value".to_string())
        );

        // Custom TTY settings should be respected
        assert!(snapshot.is_tty_stdin());
        assert!(!snapshot.is_tty_stdout());
        assert!(snapshot.is_tty_stderr());
    }

    #[test]
    fn test_create_env_snapshot_empty() {
        let snapshot = create_env_snapshot(vec![]);

        assert_eq!(snapshot.get_env("ANY_KEY"), None);
        assert!(!snapshot.is_tty_stdin());
        assert!(!snapshot.is_tty_stdout());
        assert!(!snapshot.is_tty_stderr());
    }
}
