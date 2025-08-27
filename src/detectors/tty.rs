/// Enum-based TTY detector for optimal performance and simple implementation
#[derive(Debug, Clone)]
pub enum TtyDetector {
    Real,
    Mock {
        stdin: bool,
        stdout: bool,
        stderr: bool,
    },
}

impl TtyDetector {
    /// Create a real TTY detector that uses system calls
    pub fn real() -> Self {
        Self::Real
    }

    /// Create a mock TTY detector with specified values
    pub fn mock(stdin: bool, stdout: bool, stderr: bool) -> Self {
        Self::Mock {
            stdin,
            stdout,
            stderr,
        }
    }

    /// Create a mock TTY detector for all TTY streams
    pub fn mock_all_tty() -> Self {
        Self::Mock {
            stdin: true,
            stdout: true,
            stderr: true,
        }
    }

    /// Create a mock TTY detector for no TTY streams
    pub fn mock_no_tty() -> Self {
        Self::Mock {
            stdin: false,
            stdout: false,
            stderr: false,
        }
    }

    /// Create a mock TTY detector for piped I/O (stdin TTY, stdout/stderr not)
    pub fn mock_piped_io() -> Self {
        Self::Mock {
            stdin: true,
            stdout: false,
            stderr: false,
        }
    }

    /// Check if stdin is a TTY
    pub fn is_tty_stdin(&self) -> bool {
        match self {
            Self::Real => {
                use std::io::IsTerminal;
                std::io::stdin().is_terminal()
            }
            Self::Mock { stdin, .. } => *stdin,
        }
    }

    /// Check if stdout is a TTY
    pub fn is_tty_stdout(&self) -> bool {
        match self {
            Self::Real => {
                use std::io::IsTerminal;
                std::io::stdout().is_terminal()
            }
            Self::Mock { stdout, .. } => *stdout,
        }
    }

    /// Check if stderr is a TTY
    pub fn is_tty_stderr(&self) -> bool {
        match self {
            Self::Real => {
                use std::io::IsTerminal;
                std::io::stderr().is_terminal()
            }
            Self::Mock { stderr, .. } => *stderr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tty_detector_enum() {
        let detector = TtyDetector::mock(true, false, true);
        assert!(detector.is_tty_stdin());
        assert!(!detector.is_tty_stdout());
        assert!(detector.is_tty_stderr());
    }

    #[test]
    fn test_tty_detector_convenience_methods() {
        let all_tty = TtyDetector::mock_all_tty();
        assert!(all_tty.is_tty_stdin());
        assert!(all_tty.is_tty_stdout());
        assert!(all_tty.is_tty_stderr());

        let no_tty = TtyDetector::mock_no_tty();
        assert!(!no_tty.is_tty_stdin());
        assert!(!no_tty.is_tty_stdout());
        assert!(!no_tty.is_tty_stderr());

        let piped = TtyDetector::mock_piped_io();
        assert!(piped.is_tty_stdin());
        assert!(!piped.is_tty_stdout());
        assert!(!piped.is_tty_stderr());
    }

    #[test]
    fn test_real_tty_detector_creation() {
        let detector = TtyDetector::real();
        // We can't easily test the real implementation in unit tests,
        // but we can verify it doesn't panic
        let _stdin = detector.is_tty_stdin();
        let _stdout = detector.is_tty_stdout();
        let _stderr = detector.is_tty_stderr();
    }
}
