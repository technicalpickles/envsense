use super::stream::StreamInfo;

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq, Eq,
)]
#[serde(rename_all = "lowercase")]
pub enum ColorLevel {
    None,
    Ansi16,
    Ansi256,
    Truecolor,
}

/// Traits specific to terminal capabilities and stream information
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq, Eq,
)]
pub struct TerminalTraits {
    /// Whether the terminal is interactive (both stdin and stdout are TTYs)
    pub interactive: bool,
    /// The color support level of the terminal
    pub color_level: ColorLevel,
    /// Information about the stdin stream
    pub stdin: StreamInfo,
    /// Information about the stdout stream
    pub stdout: StreamInfo,
    /// Information about the stderr stream
    pub stderr: StreamInfo,
    /// Whether the terminal supports hyperlinks
    pub supports_hyperlinks: bool,
}

fn level_from_flags(has_basic: bool, has_256: bool, has_16m: bool) -> ColorLevel {
    if has_16m {
        ColorLevel::Truecolor
    } else if has_256 {
        ColorLevel::Ansi256
    } else if has_basic {
        ColorLevel::Ansi16
    } else {
        ColorLevel::None
    }
}

fn map_color_level(level: Option<supports_color::ColorLevel>) -> ColorLevel {
    match level {
        Some(l) => level_from_flags(l.has_basic, l.has_256, l.has_16m),
        None => ColorLevel::None,
    }
}

impl Default for TerminalTraits {
    fn default() -> Self {
        Self {
            interactive: false,
            color_level: ColorLevel::None,
            stdin: StreamInfo::default(),
            stdout: StreamInfo::default(),
            stderr: StreamInfo::default(),
            supports_hyperlinks: false,
        }
    }
}

impl TerminalTraits {
    /// Detect terminal traits from the current environment
    pub fn detect() -> Self {
        let stdin = StreamInfo::stdin();
        let stdout = StreamInfo::stdout();
        let stderr = StreamInfo::stderr();
        let interactive = stdin.tty && stdout.tty;
        let color_level = map_color_level(supports_color::on(supports_color::Stream::Stdout));
        let supports_hyperlinks = supports_hyperlinks::on(supports_hyperlinks::Stream::Stdout);

        Self {
            interactive,
            color_level,
            stdin,
            stdout,
            stderr,
            supports_hyperlinks,
        }
    }

    // Legacy compatibility methods for backward compatibility
    #[deprecated(note = "Use interactive field instead")]
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    #[deprecated(note = "Use stdin.tty field instead")]
    pub fn is_tty_stdin(&self) -> bool {
        self.stdin.tty
    }

    #[deprecated(note = "Use stdout.tty field instead")]
    pub fn is_tty_stdout(&self) -> bool {
        self.stdout.tty
    }

    #[deprecated(note = "Use stderr.tty field instead")]
    pub fn is_tty_stderr(&self) -> bool {
        self.stderr.tty
    }

    #[deprecated(note = "Use stdin.piped field instead")]
    pub fn is_piped_stdin(&self) -> bool {
        self.stdin.piped
    }

    #[deprecated(note = "Use stdout.piped field instead")]
    pub fn is_piped_stdout(&self) -> bool {
        self.stdout.piped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_color_level() {
        assert_eq!(level_from_flags(true, false, false), ColorLevel::Ansi16);
        assert_eq!(level_from_flags(true, true, false), ColorLevel::Ansi256);
        assert_eq!(level_from_flags(true, true, true), ColorLevel::Truecolor);
        assert_eq!(level_from_flags(false, false, false), ColorLevel::None);
    }

    #[test]
    fn default_terminal_traits() {
        let traits = TerminalTraits::default();
        assert!(!traits.interactive);
        assert_eq!(traits.color_level, ColorLevel::None);
        assert!(!traits.stdin.tty);
        assert!(traits.stdin.piped);
        assert!(!traits.stdout.tty);
        assert!(traits.stdout.piped);
        assert!(!traits.stderr.tty);
        assert!(traits.stderr.piped);
        assert!(!traits.supports_hyperlinks);
    }

    #[test]
    fn terminal_traits_serialization() {
        let traits = TerminalTraits {
            interactive: true,
            color_level: ColorLevel::Truecolor,
            stdin: StreamInfo {
                tty: true,
                piped: false,
            },
            stdout: StreamInfo {
                tty: true,
                piped: false,
            },
            stderr: StreamInfo {
                tty: true,
                piped: false,
            },
            supports_hyperlinks: true,
        };
        let json = serde_json::to_string(&traits).unwrap();
        assert!(json.contains("\"interactive\":true"));
        assert!(json.contains("\"color_level\":\"truecolor\""));
        assert!(json.contains("\"stdin\":{\"tty\":true,\"piped\":false}"));
        assert!(json.contains("\"stdout\":{\"tty\":true,\"piped\":false}"));
        assert!(json.contains("\"stderr\":{\"tty\":true,\"piped\":false}"));
        assert!(json.contains("\"supports_hyperlinks\":true"));
    }

    #[test]
    fn legacy_compatibility_methods() {
        let traits = TerminalTraits {
            interactive: true,
            color_level: ColorLevel::Ansi256,
            stdin: StreamInfo {
                tty: true,
                piped: false,
            },
            stdout: StreamInfo {
                tty: true,
                piped: false,
            },
            stderr: StreamInfo {
                tty: false,
                piped: true,
            },
            supports_hyperlinks: false,
        };

        assert!(traits.is_interactive());
        assert!(traits.is_tty_stdin());
        assert!(traits.is_tty_stdout());
        assert!(!traits.is_tty_stderr());
        assert!(!traits.is_piped_stdin());
        assert!(!traits.is_piped_stdout());
    }
}
