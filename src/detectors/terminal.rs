use crate::detectors::{Detection, Detector, EnvSnapshot};
use crate::traits::terminal::{ColorLevel, TerminalTraits};
use serde_json::json;

pub struct TerminalDetector;

impl TerminalDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Detector for TerminalDetector {
    fn name(&self) -> &'static str {
        "terminal"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        // Use existing terminal traits detection
        let terminal_traits = TerminalTraits::detect();

        // Convert to traits patches
        detection.traits_patch.insert(
            "is_interactive".to_string(),
            json!(terminal_traits.is_interactive),
        );
        detection
            .traits_patch
            .insert("is_tty_stdin".to_string(), json!(snap.is_tty_stdin));
        detection
            .traits_patch
            .insert("is_tty_stdout".to_string(), json!(snap.is_tty_stdout));
        detection
            .traits_patch
            .insert("is_tty_stderr".to_string(), json!(snap.is_tty_stderr));
        detection
            .traits_patch
            .insert("is_piped_stdin".to_string(), json!(!snap.is_tty_stdin));
        detection
            .traits_patch
            .insert("is_piped_stdout".to_string(), json!(!snap.is_tty_stdout));
        detection.traits_patch.insert(
            "supports_hyperlinks".to_string(),
            json!(terminal_traits.supports_hyperlinks),
        );

        // Map color level enum to JSON
        let color_level_str = match terminal_traits.color_level {
            ColorLevel::None => "none",
            ColorLevel::Ansi16 => "ansi16",
            ColorLevel::Ansi256 => "ansi256",
            ColorLevel::Truecolor => "truecolor",
        };
        detection
            .traits_patch
            .insert("color_level".to_string(), json!(color_level_str));

        detection.confidence = 1.0; // Terminal detection is always reliable

        detection
    }
}

impl Default for TerminalDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_env_snapshot(
        is_tty_stdin: bool,
        is_tty_stdout: bool,
        is_tty_stderr: bool,
    ) -> EnvSnapshot {
        EnvSnapshot {
            env_vars: HashMap::new(),
            is_tty_stdin,
            is_tty_stdout,
            is_tty_stderr,
        }
    }

    #[test]
    fn detects_terminal_traits() {
        let detector = TerminalDetector::new();
        let snapshot = create_env_snapshot(true, true, false);

        let detection = detector.detect(&snapshot);

        // Should have terminal traits
        assert!(detection.traits_patch.contains_key("is_tty_stdin"));
        assert!(detection.traits_patch.contains_key("is_tty_stdout"));
        assert!(detection.traits_patch.contains_key("is_tty_stderr"));
        assert!(detection.traits_patch.contains_key("is_piped_stdin"));
        assert!(detection.traits_patch.contains_key("is_piped_stdout"));
        assert!(detection.traits_patch.contains_key("color_level"));
        assert!(detection.traits_patch.contains_key("supports_hyperlinks"));
        assert!(detection.traits_patch.contains_key("is_interactive"));

        // TTY values should match snapshot
        assert_eq!(
            detection.traits_patch.get("is_tty_stdin").unwrap(),
            &json!(true)
        );
        assert_eq!(
            detection.traits_patch.get("is_tty_stdout").unwrap(),
            &json!(true)
        );
        assert_eq!(
            detection.traits_patch.get("is_tty_stderr").unwrap(),
            &json!(false)
        );
        assert_eq!(
            detection.traits_patch.get("is_piped_stdin").unwrap(),
            &json!(false)
        );
        assert_eq!(
            detection.traits_patch.get("is_piped_stdout").unwrap(),
            &json!(false)
        );

        assert_eq!(detection.confidence, 1.0);
    }

    #[test]
    fn detects_piped_io() {
        let detector = TerminalDetector::new();
        let snapshot = create_env_snapshot(false, false, false);

        let detection = detector.detect(&snapshot);

        // All should be piped, not TTY
        assert_eq!(
            detection.traits_patch.get("is_tty_stdin").unwrap(),
            &json!(false)
        );
        assert_eq!(
            detection.traits_patch.get("is_tty_stdout").unwrap(),
            &json!(false)
        );
        assert_eq!(
            detection.traits_patch.get("is_tty_stderr").unwrap(),
            &json!(false)
        );
        assert_eq!(
            detection.traits_patch.get("is_piped_stdin").unwrap(),
            &json!(true)
        );
        assert_eq!(
            detection.traits_patch.get("is_piped_stdout").unwrap(),
            &json!(true)
        );
    }
}
