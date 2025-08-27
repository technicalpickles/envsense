use crate::detectors::{Detection, Detector, EnvSnapshot, confidence::TERMINAL};
use crate::schema::Evidence;
use crate::traits::terminal::ColorLevel;
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

        // TTY detection is always reliable
        detection.confidence = TERMINAL;

        // Use TTY values from snapshot (now via dependency injection)
        let is_interactive = snap.is_tty_stdin() && snap.is_tty_stdout();

        // Detect color level and hyperlinks support, but allow override
        let color_level = if let Some(override_color) = snap.env_vars.get("ENVSENSE_COLOR_LEVEL") {
            match override_color.as_str() {
                "none" => ColorLevel::None,
                "ansi16" => ColorLevel::Ansi16,
                "ansi256" => ColorLevel::Ansi256,
                "truecolor" => ColorLevel::Truecolor,
                _ => ColorLevel::None,
            }
        } else {
            // Use runtime detection
            let level = supports_color::on(supports_color::Stream::Stdout);
            match level {
                Some(l) => {
                    if l.has_16m {
                        ColorLevel::Truecolor
                    } else if l.has_256 {
                        ColorLevel::Ansi256
                    } else if l.has_basic {
                        ColorLevel::Ansi16
                    } else {
                        ColorLevel::None
                    }
                }
                None => ColorLevel::None,
            }
        };

        let supports_hyperlinks = snap
            .env_vars
            .get("ENVSENSE_SUPPORTS_HYPERLINKS")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or_else(|| supports_hyperlinks::on(supports_hyperlinks::Stream::Stdout));

        // Convert to traits patches
        detection
            .traits_patch
            .insert("is_interactive".to_string(), json!(is_interactive));
        detection
            .traits_patch
            .insert("is_tty_stdin".to_string(), json!(snap.is_tty_stdin()));
        detection
            .traits_patch
            .insert("is_tty_stdout".to_string(), json!(snap.is_tty_stdout()));
        detection
            .traits_patch
            .insert("is_tty_stderr".to_string(), json!(snap.is_tty_stderr()));
        detection
            .traits_patch
            .insert("is_piped_stdin".to_string(), json!(!snap.is_tty_stdin()));
        detection
            .traits_patch
            .insert("is_piped_stdout".to_string(), json!(!snap.is_tty_stdout()));
        detection.traits_patch.insert(
            "supports_hyperlinks".to_string(),
            json!(supports_hyperlinks),
        );

        // Map color level enum to JSON
        let color_level_str = match color_level {
            ColorLevel::None => "none",
            ColorLevel::Ansi16 => "ansi16",
            ColorLevel::Ansi256 => "ansi256",
            ColorLevel::Truecolor => "truecolor",
        };
        detection
            .traits_patch
            .insert("color_level".to_string(), json!(color_level_str));

        // Add evidence for TTY detection
        detection.evidence.push(
            Evidence::tty_trait("is_tty_stdin", snap.is_tty_stdin())
                .with_supports(vec!["is_tty_stdin".into()])
                .with_confidence(TERMINAL)
        );
        detection.evidence.push(
            Evidence::tty_trait("is_tty_stdout", snap.is_tty_stdout())
                .with_supports(vec!["is_tty_stdout".into()])
                .with_confidence(TERMINAL)
        );
        detection.evidence.push(
            Evidence::tty_trait("is_tty_stderr", snap.is_tty_stderr())
                .with_supports(vec!["is_tty_stderr".into()])
                .with_confidence(TERMINAL)
        );

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
        EnvSnapshot::with_mock_tty(HashMap::new(), is_tty_stdin, is_tty_stdout, is_tty_stderr)
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
