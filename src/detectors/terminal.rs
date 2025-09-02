use crate::detectors::{Detection, Detector, EnvSnapshot, confidence::TERMINAL};
use crate::schema::Evidence;
use crate::traits::stream::StreamInfo;
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
        let mut detection = Detection {
            confidence: TERMINAL,
            ..Default::default()
        };

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

        // Create nested TerminalTraits object
        let terminal_traits = TerminalTraits {
            interactive: is_interactive,
            color_level,
            stdin: StreamInfo::from_tty(snap.is_tty_stdin()),
            stdout: StreamInfo::from_tty(snap.is_tty_stdout()),
            stderr: StreamInfo::from_tty(snap.is_tty_stderr()),
            supports_hyperlinks,
        };

        // Insert as nested object under "terminal" key
        detection.traits_patch.insert(
            "terminal".to_string(),
            serde_json::to_value(terminal_traits.clone()).unwrap(),
        );

        // Maintain backward compatibility with legacy flat keys
        detection.traits_patch.insert(
            "is_interactive".to_string(),
            json!(terminal_traits.interactive),
        );
        detection
            .traits_patch
            .insert("is_tty_stdin".to_string(), json!(terminal_traits.stdin.tty));
        detection.traits_patch.insert(
            "is_tty_stdout".to_string(),
            json!(terminal_traits.stdout.tty),
        );
        detection.traits_patch.insert(
            "is_tty_stderr".to_string(),
            json!(terminal_traits.stderr.tty),
        );
        detection.traits_patch.insert(
            "is_piped_stdin".to_string(),
            json!(terminal_traits.stdin.piped),
        );
        detection.traits_patch.insert(
            "is_piped_stdout".to_string(),
            json!(terminal_traits.stdout.piped),
        );
        detection.traits_patch.insert(
            "supports_hyperlinks".to_string(),
            json!(terminal_traits.supports_hyperlinks),
        );

        // Map color level enum to JSON for backward compatibility
        let color_level_str = match terminal_traits.color_level {
            ColorLevel::None => "none",
            ColorLevel::Ansi16 => "ansi16",
            ColorLevel::Ansi256 => "ansi256",
            ColorLevel::Truecolor => "truecolor",
        };
        detection
            .traits_patch
            .insert("color_level".to_string(), json!(color_level_str));

        // Add evidence for TTY detection with nested field paths
        detection.evidence.push(
            Evidence::tty_trait("terminal.stdin.tty", terminal_traits.stdin.tty)
                .with_supports(vec!["terminal.stdin.tty".into()])
                .with_confidence(TERMINAL),
        );
        detection.evidence.push(
            Evidence::tty_trait("terminal.stdout.tty", terminal_traits.stdout.tty)
                .with_supports(vec!["terminal.stdout.tty".into()])
                .with_confidence(TERMINAL),
        );
        detection.evidence.push(
            Evidence::tty_trait("terminal.stderr.tty", terminal_traits.stderr.tty)
                .with_supports(vec!["terminal.stderr.tty".into()])
                .with_confidence(TERMINAL),
        );

        // Add evidence for interactive detection
        detection.evidence.push(
            Evidence::tty_trait("terminal.interactive", terminal_traits.interactive)
                .with_supports(vec!["terminal.interactive".into()])
                .with_confidence(TERMINAL),
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

    use crate::detectors::test_utils::create_env_snapshot_with_tty;

    #[test]
    fn detects_terminal_traits() {
        let detector = TerminalDetector::new();
        let snapshot = create_env_snapshot_with_tty(vec![], true, true, false);

        let detection = detector.detect(&snapshot);

        // Should have nested terminal object
        assert!(detection.traits_patch.contains_key("terminal"));

        // Verify nested structure
        let terminal_obj = detection
            .traits_patch
            .get("terminal")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(terminal_obj.get("interactive").unwrap(), &json!(true));

        let stdin_obj = terminal_obj.get("stdin").unwrap().as_object().unwrap();
        assert_eq!(stdin_obj.get("tty").unwrap(), &json!(true));
        assert_eq!(stdin_obj.get("piped").unwrap(), &json!(false));

        let stdout_obj = terminal_obj.get("stdout").unwrap().as_object().unwrap();
        assert_eq!(stdout_obj.get("tty").unwrap(), &json!(true));
        assert_eq!(stdout_obj.get("piped").unwrap(), &json!(false));

        let stderr_obj = terminal_obj.get("stderr").unwrap().as_object().unwrap();
        assert_eq!(stderr_obj.get("tty").unwrap(), &json!(false));
        assert_eq!(stderr_obj.get("piped").unwrap(), &json!(true));

        // Should have backward compatibility flat keys
        assert!(detection.traits_patch.contains_key("is_tty_stdin"));
        assert!(detection.traits_patch.contains_key("is_tty_stdout"));
        assert!(detection.traits_patch.contains_key("is_tty_stderr"));
        assert!(detection.traits_patch.contains_key("is_piped_stdin"));
        assert!(detection.traits_patch.contains_key("is_piped_stdout"));
        assert!(detection.traits_patch.contains_key("color_level"));
        assert!(detection.traits_patch.contains_key("supports_hyperlinks"));
        assert!(detection.traits_patch.contains_key("is_interactive"));

        // Legacy TTY values should match snapshot
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
        let snapshot = create_env_snapshot_with_tty(vec![], false, false, false);

        let detection = detector.detect(&snapshot);

        // Should have nested terminal object
        assert!(detection.traits_patch.contains_key("terminal"));

        // Verify nested structure shows all piped
        let terminal_obj = detection
            .traits_patch
            .get("terminal")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(terminal_obj.get("interactive").unwrap(), &json!(false));

        let stdin_obj = terminal_obj.get("stdin").unwrap().as_object().unwrap();
        assert_eq!(stdin_obj.get("tty").unwrap(), &json!(false));
        assert_eq!(stdin_obj.get("piped").unwrap(), &json!(true));

        let stdout_obj = terminal_obj.get("stdout").unwrap().as_object().unwrap();
        assert_eq!(stdout_obj.get("tty").unwrap(), &json!(false));
        assert_eq!(stdout_obj.get("piped").unwrap(), &json!(true));

        let stderr_obj = terminal_obj.get("stderr").unwrap().as_object().unwrap();
        assert_eq!(stderr_obj.get("tty").unwrap(), &json!(false));
        assert_eq!(stderr_obj.get("piped").unwrap(), &json!(true));

        // Legacy flat keys should also show all piped, not TTY
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

    #[test]
    fn nested_terminal_traits_json_serialization() {
        let detector = TerminalDetector::new();
        let snapshot = create_env_snapshot_with_tty(vec![], true, true, false);

        let detection = detector.detect(&snapshot);

        // Verify JSON serialization of nested structure
        let terminal_value = detection.traits_patch.get("terminal").unwrap();
        let json_str = serde_json::to_string(terminal_value).unwrap();

        // Should contain nested structure
        assert!(json_str.contains("\"interactive\":true"));
        assert!(json_str.contains("\"tty\":true"));
        assert!(json_str.contains("\"piped\":false"));
        assert!(json_str.contains("\"color_level\":"));
        assert!(json_str.contains("\"supports_hyperlinks\":"));

        // Verify it can be deserialized back to TerminalTraits
        let terminal_traits: TerminalTraits =
            serde_json::from_value(terminal_value.clone()).unwrap();
        assert!(terminal_traits.interactive);
        assert!(terminal_traits.stdin.tty);
        assert!(!terminal_traits.stdin.piped);
        assert!(terminal_traits.stdout.tty);
        assert!(!terminal_traits.stdout.piped);
        assert!(!terminal_traits.stderr.tty);
        assert!(terminal_traits.stderr.piped);
    }

    #[test]
    fn evidence_uses_nested_field_paths() {
        let detector = TerminalDetector::new();
        let snapshot = create_env_snapshot_with_tty(vec![], true, false, true);

        let detection = detector.detect(&snapshot);

        // Verify evidence uses nested field paths
        let evidence_supports: Vec<String> = detection
            .evidence
            .iter()
            .flat_map(|e| e.supports.clone())
            .collect();

        assert!(evidence_supports.contains(&"terminal.stdin.tty".to_string()));
        assert!(evidence_supports.contains(&"terminal.stdout.tty".to_string()));
        assert!(evidence_supports.contains(&"terminal.stderr.tty".to_string()));
        assert!(evidence_supports.contains(&"terminal.interactive".to_string()));

        // Verify evidence count (should have 4 evidence items)
        assert_eq!(detection.evidence.len(), 4);
    }

    #[test]
    fn color_level_override_works_with_nested_structure() {
        let detector = TerminalDetector::new();
        let env_vars = vec![("ENVSENSE_COLOR_LEVEL", "truecolor")];
        let snapshot = create_env_snapshot_with_tty(env_vars, true, true, false);

        let detection = detector.detect(&snapshot);

        // Verify nested structure has correct color level
        let terminal_obj = detection
            .traits_patch
            .get("terminal")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(
            terminal_obj.get("color_level").unwrap(),
            &json!("truecolor")
        );

        // Verify legacy flat key also has correct color level
        assert_eq!(
            detection.traits_patch.get("color_level").unwrap(),
            &json!("truecolor")
        );
    }

    #[test]
    fn hyperlinks_override_works_with_nested_structure() {
        let detector = TerminalDetector::new();
        let env_vars = vec![("ENVSENSE_SUPPORTS_HYPERLINKS", "true")];
        let snapshot = create_env_snapshot_with_tty(env_vars, false, false, false);

        let detection = detector.detect(&snapshot);

        // Verify nested structure has correct hyperlinks support
        let terminal_obj = detection
            .traits_patch
            .get("terminal")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(
            terminal_obj.get("supports_hyperlinks").unwrap(),
            &json!(true)
        );

        // Verify legacy flat key also has correct hyperlinks support
        assert_eq!(
            detection.traits_patch.get("supports_hyperlinks").unwrap(),
            &json!(true)
        );
    }

    #[test]
    fn all_color_levels_work_with_nested_structure() {
        let detector = TerminalDetector::new();

        // Test each color level
        let test_cases = vec![
            ("none", ColorLevel::None),
            ("ansi16", ColorLevel::Ansi16),
            ("ansi256", ColorLevel::Ansi256),
            ("truecolor", ColorLevel::Truecolor),
        ];

        for (level_str, expected_level) in test_cases {
            let env_vars = vec![("ENVSENSE_COLOR_LEVEL", level_str)];
            let snapshot = create_env_snapshot_with_tty(env_vars, true, true, false);
            let detection = detector.detect(&snapshot);

            // Verify nested structure
            let terminal_obj = detection
                .traits_patch
                .get("terminal")
                .unwrap()
                .as_object()
                .unwrap();
            assert_eq!(terminal_obj.get("color_level").unwrap(), &json!(level_str));

            // Verify deserialization
            let terminal_value = detection.traits_patch.get("terminal").unwrap();
            let terminal_traits: TerminalTraits =
                serde_json::from_value(terminal_value.clone()).unwrap();
            assert_eq!(terminal_traits.color_level, expected_level);
        }
    }

    #[test]
    fn invalid_color_level_defaults_to_none() {
        let detector = TerminalDetector::new();

        let invalid_values = vec!["invalid", "TRUECOLOR", "256", "", "null"];

        for invalid_value in invalid_values {
            let env_vars = vec![("ENVSENSE_COLOR_LEVEL", invalid_value)];
            let snapshot = create_env_snapshot_with_tty(env_vars, true, true, false);
            let detection = detector.detect(&snapshot);

            // Should default to "none"
            let terminal_obj = detection
                .traits_patch
                .get("terminal")
                .unwrap()
                .as_object()
                .unwrap();
            assert_eq!(terminal_obj.get("color_level").unwrap(), &json!("none"));

            let terminal_value = detection.traits_patch.get("terminal").unwrap();
            let terminal_traits: TerminalTraits =
                serde_json::from_value(terminal_value.clone()).unwrap();
            assert_eq!(terminal_traits.color_level, ColorLevel::None);
        }
    }

    #[test]
    fn invalid_hyperlinks_values_handled_gracefully() {
        let detector = TerminalDetector::new();

        // First, get the runtime detection value
        let snapshot_no_env = create_env_snapshot_with_tty(vec![], true, true, false);
        let detection_no_env = detector.detect(&snapshot_no_env);
        let runtime_hyperlinks = detection_no_env
            .traits_patch
            .get("terminal")
            .unwrap()
            .as_object()
            .unwrap()
            .get("supports_hyperlinks")
            .unwrap()
            .as_bool()
            .unwrap();

        let invalid_values = vec!["invalid", "1", "yes", "TRUE", ""];

        for invalid_value in invalid_values {
            let env_vars = vec![("ENVSENSE_SUPPORTS_HYPERLINKS", invalid_value)];
            let snapshot = create_env_snapshot_with_tty(env_vars, true, true, false);
            let detection = detector.detect(&snapshot);

            // Should fall back to runtime detection
            let terminal_obj = detection
                .traits_patch
                .get("terminal")
                .unwrap()
                .as_object()
                .unwrap();
            assert_eq!(
                terminal_obj.get("supports_hyperlinks").unwrap(),
                &json!(runtime_hyperlinks),
                "Failed for invalid hyperlinks value: {}",
                invalid_value
            );
        }
    }

    #[test]
    fn mixed_stream_scenarios() {
        let detector = TerminalDetector::new();

        // Test various combinations of TTY states
        let test_cases = vec![
            // (stdin, stdout, stderr, expected_interactive)
            (true, true, true, true),     // All TTY
            (true, true, false, true),    // Interactive (stdin+stdout TTY)
            (true, false, true, false),   // stdin TTY, stdout piped
            (false, true, true, false),   // stdin piped, stdout TTY
            (false, false, false, false), // All piped
            (true, false, false, false),  // Only stdin TTY
            (false, true, false, false),  // Only stdout TTY
            (false, false, true, false),  // Only stderr TTY
        ];

        for (stdin_tty, stdout_tty, stderr_tty, expected_interactive) in test_cases {
            let snapshot = create_env_snapshot_with_tty(vec![], stdin_tty, stdout_tty, stderr_tty);
            let detection = detector.detect(&snapshot);

            let terminal_obj = detection
                .traits_patch
                .get("terminal")
                .unwrap()
                .as_object()
                .unwrap();
            assert_eq!(
                terminal_obj.get("interactive").unwrap(),
                &json!(expected_interactive),
                "Failed for stdin={}, stdout={}, stderr={}",
                stdin_tty,
                stdout_tty,
                stderr_tty
            );

            // Verify individual stream states
            let stdin_obj = terminal_obj.get("stdin").unwrap().as_object().unwrap();
            assert_eq!(stdin_obj.get("tty").unwrap(), &json!(stdin_tty));
            assert_eq!(stdin_obj.get("piped").unwrap(), &json!(!stdin_tty));

            let stdout_obj = terminal_obj.get("stdout").unwrap().as_object().unwrap();
            assert_eq!(stdout_obj.get("tty").unwrap(), &json!(stdout_tty));
            assert_eq!(stdout_obj.get("piped").unwrap(), &json!(!stdout_tty));

            let stderr_obj = terminal_obj.get("stderr").unwrap().as_object().unwrap();
            assert_eq!(stderr_obj.get("tty").unwrap(), &json!(stderr_tty));
            assert_eq!(stderr_obj.get("piped").unwrap(), &json!(!stderr_tty));
        }
    }

    #[test]
    fn nested_and_flat_keys_consistency() {
        let detector = TerminalDetector::new();
        let snapshot = create_env_snapshot_with_tty(vec![], true, false, true);
        let detection = detector.detect(&snapshot);

        // Get nested values
        let terminal_obj = detection
            .traits_patch
            .get("terminal")
            .unwrap()
            .as_object()
            .unwrap();
        let nested_interactive = terminal_obj.get("interactive").unwrap().as_bool().unwrap();
        let nested_stdin_tty = terminal_obj
            .get("stdin")
            .unwrap()
            .as_object()
            .unwrap()
            .get("tty")
            .unwrap()
            .as_bool()
            .unwrap();
        let nested_stdout_tty = terminal_obj
            .get("stdout")
            .unwrap()
            .as_object()
            .unwrap()
            .get("tty")
            .unwrap()
            .as_bool()
            .unwrap();
        let nested_stderr_tty = terminal_obj
            .get("stderr")
            .unwrap()
            .as_object()
            .unwrap()
            .get("tty")
            .unwrap()
            .as_bool()
            .unwrap();
        let nested_stdin_piped = terminal_obj
            .get("stdin")
            .unwrap()
            .as_object()
            .unwrap()
            .get("piped")
            .unwrap()
            .as_bool()
            .unwrap();
        let nested_stdout_piped = terminal_obj
            .get("stdout")
            .unwrap()
            .as_object()
            .unwrap()
            .get("piped")
            .unwrap()
            .as_bool()
            .unwrap();
        let nested_color_level = terminal_obj.get("color_level").unwrap().as_str().unwrap();
        let nested_hyperlinks = terminal_obj
            .get("supports_hyperlinks")
            .unwrap()
            .as_bool()
            .unwrap();

        // Get flat values
        let flat_interactive = detection
            .traits_patch
            .get("is_interactive")
            .unwrap()
            .as_bool()
            .unwrap();
        let flat_stdin_tty = detection
            .traits_patch
            .get("is_tty_stdin")
            .unwrap()
            .as_bool()
            .unwrap();
        let flat_stdout_tty = detection
            .traits_patch
            .get("is_tty_stdout")
            .unwrap()
            .as_bool()
            .unwrap();
        let flat_stderr_tty = detection
            .traits_patch
            .get("is_tty_stderr")
            .unwrap()
            .as_bool()
            .unwrap();
        let flat_stdin_piped = detection
            .traits_patch
            .get("is_piped_stdin")
            .unwrap()
            .as_bool()
            .unwrap();
        let flat_stdout_piped = detection
            .traits_patch
            .get("is_piped_stdout")
            .unwrap()
            .as_bool()
            .unwrap();
        let flat_color_level = detection
            .traits_patch
            .get("color_level")
            .unwrap()
            .as_str()
            .unwrap();
        let flat_hyperlinks = detection
            .traits_patch
            .get("supports_hyperlinks")
            .unwrap()
            .as_bool()
            .unwrap();

        // Verify consistency
        assert_eq!(nested_interactive, flat_interactive);
        assert_eq!(nested_stdin_tty, flat_stdin_tty);
        assert_eq!(nested_stdout_tty, flat_stdout_tty);
        assert_eq!(nested_stderr_tty, flat_stderr_tty);
        assert_eq!(nested_stdin_piped, flat_stdin_piped);
        assert_eq!(nested_stdout_piped, flat_stdout_piped);
        assert_eq!(nested_color_level, flat_color_level);
        assert_eq!(nested_hyperlinks, flat_hyperlinks);
    }

    #[test]
    fn evidence_content_and_confidence_validation() {
        let detector = TerminalDetector::new();
        let snapshot = create_env_snapshot_with_tty(vec![], true, false, true);
        let detection = detector.detect(&snapshot);

        // Verify all evidence has correct confidence
        for evidence in &detection.evidence {
            assert_eq!(evidence.confidence, TERMINAL);
        }

        // Verify evidence supports correct fields
        let evidence_by_supports: std::collections::HashMap<String, &Evidence> = detection
            .evidence
            .iter()
            .filter_map(|e| e.supports.first().map(|s| (s.clone(), e)))
            .collect();

        assert!(evidence_by_supports.contains_key("terminal.stdin.tty"));
        assert!(evidence_by_supports.contains_key("terminal.stdout.tty"));
        assert!(evidence_by_supports.contains_key("terminal.stderr.tty"));
        assert!(evidence_by_supports.contains_key("terminal.interactive"));

        // Verify evidence keys are meaningful
        for evidence in &detection.evidence {
            assert!(!evidence.key.is_empty());
        }
    }

    #[test]
    fn hyperlinks_boolean_parsing_edge_cases() {
        let detector = TerminalDetector::new();

        // First, get the runtime detection value for fallback cases
        let snapshot_no_env = create_env_snapshot_with_tty(vec![], true, true, false);
        let detection_no_env = detector.detect(&snapshot_no_env);
        let runtime_hyperlinks = detection_no_env
            .traits_patch
            .get("terminal")
            .unwrap()
            .as_object()
            .unwrap()
            .get("supports_hyperlinks")
            .unwrap()
            .as_bool()
            .unwrap();

        let test_cases = vec![
            ("true", true),
            ("false", false),
            ("True", runtime_hyperlinks), // Case sensitive, should fall back to runtime
            ("FALSE", runtime_hyperlinks), // Case sensitive, should fall back to runtime
            ("0", runtime_hyperlinks),    // Not a valid boolean, should fall back
            ("1", runtime_hyperlinks),    // Not a valid boolean, should fall back
        ];

        for (value, expected) in test_cases {
            let env_vars = vec![("ENVSENSE_SUPPORTS_HYPERLINKS", value)];
            let snapshot = create_env_snapshot_with_tty(env_vars, true, true, false);
            let detection = detector.detect(&snapshot);

            let terminal_obj = detection
                .traits_patch
                .get("terminal")
                .unwrap()
                .as_object()
                .unwrap();
            assert_eq!(
                terminal_obj.get("supports_hyperlinks").unwrap(),
                &json!(expected),
                "Failed for hyperlinks value: {}",
                value
            );
        }
    }

    #[test]
    fn detector_name_and_default_implementation() {
        let detector = TerminalDetector::new();
        assert_eq!(detector.name(), "terminal");

        // Test Default implementation
        let default_detector = TerminalDetector::default();
        assert_eq!(default_detector.name(), "terminal");
    }

    #[test]
    fn confidence_level_consistency() {
        let detector = TerminalDetector::new();
        let snapshot = create_env_snapshot_with_tty(vec![], true, true, false);
        let detection = detector.detect(&snapshot);

        // Verify detection confidence
        assert_eq!(detection.confidence, TERMINAL);

        // Verify all evidence has same confidence
        for evidence in &detection.evidence {
            assert_eq!(evidence.confidence, TERMINAL);
        }
    }

    #[test]
    fn empty_environment_variables_handling() {
        let detector = TerminalDetector::new();

        // First, get the runtime detection value for hyperlinks
        let snapshot_no_env = create_env_snapshot_with_tty(vec![], true, true, false);
        let detection_no_env = detector.detect(&snapshot_no_env);
        let runtime_hyperlinks = detection_no_env
            .traits_patch
            .get("terminal")
            .unwrap()
            .as_object()
            .unwrap()
            .get("supports_hyperlinks")
            .unwrap()
            .as_bool()
            .unwrap();

        // Test with empty string values
        let env_vars = vec![
            ("ENVSENSE_COLOR_LEVEL", ""),
            ("ENVSENSE_SUPPORTS_HYPERLINKS", ""),
        ];
        let snapshot = create_env_snapshot_with_tty(env_vars, true, true, false);
        let detection = detector.detect(&snapshot);

        // Should handle empty values gracefully
        let terminal_obj = detection
            .traits_patch
            .get("terminal")
            .unwrap()
            .as_object()
            .unwrap();

        // Empty color level should default to "none"
        assert_eq!(terminal_obj.get("color_level").unwrap(), &json!("none"));

        // Empty hyperlinks should fall back to runtime detection
        assert_eq!(
            terminal_obj.get("supports_hyperlinks").unwrap(),
            &json!(runtime_hyperlinks)
        );
    }
}
