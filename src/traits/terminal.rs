use is_terminal::IsTerminal;

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

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq, Eq,
)]
pub struct TerminalTraits {
    pub color_level: ColorLevel,
    pub is_interactive: bool,
    pub is_tty_stdin: bool,
    pub is_tty_stdout: bool,
    pub is_tty_stderr: bool,
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

impl TerminalTraits {
    pub fn detect() -> Self {
        let is_tty_stdin = std::io::stdin().is_terminal();
        let is_tty_stdout = std::io::stdout().is_terminal();
        let is_tty_stderr = std::io::stderr().is_terminal();
        let is_interactive = is_tty_stdin && is_tty_stdout;
        let color_level = map_color_level(supports_color::on(supports_color::Stream::Stdout));
        let supports_hyperlinks = supports_hyperlinks::on(supports_hyperlinks::Stream::Stdout);
        TerminalTraits {
            color_level,
            is_interactive,
            is_tty_stdin,
            is_tty_stdout,
            is_tty_stderr,
            supports_hyperlinks,
        }
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
    fn derives_piped_flags() {
        let traits = TerminalTraits {
            color_level: ColorLevel::None,
            is_interactive: false,
            is_tty_stdin: false,
            is_tty_stdout: true,
            is_tty_stderr: true,
            supports_hyperlinks: false,
        };
        let t: crate::schema::Traits = traits.clone().into();
        assert!(t.is_piped_stdin);
        assert!(!t.is_piped_stdout);
        assert_eq!(t.is_interactive, traits.is_interactive);
    }
}
