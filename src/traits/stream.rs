use is_terminal::IsTerminal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Information about a stream (stdin, stdout, stderr)
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
pub struct StreamInfo {
    /// Whether the stream is connected to a TTY
    pub tty: bool,
    /// Whether the stream is piped (not connected to a TTY)
    pub piped: bool,
}

impl Default for StreamInfo {
    fn default() -> Self {
        Self {
            tty: false,
            piped: true,
        }
    }
}

impl StreamInfo {
    /// Create stream info from TTY status
    pub fn from_tty(is_tty: bool) -> Self {
        Self {
            tty: is_tty,
            piped: !is_tty,
        }
    }

    /// Create stream info for stdin
    pub fn stdin() -> Self {
        Self::from_tty(std::io::stdin().is_terminal())
    }

    /// Create stream info for stdout
    pub fn stdout() -> Self {
        Self::from_tty(std::io::stdout().is_terminal())
    }

    /// Create stream info for stderr
    pub fn stderr() -> Self {
        Self::from_tty(std::io::stderr().is_terminal())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_stream_info() {
        let info = StreamInfo::default();
        assert!(!info.tty);
        assert!(info.piped);
    }

    #[test]
    fn from_tty_true() {
        let info = StreamInfo::from_tty(true);
        assert!(info.tty);
        assert!(!info.piped);
    }

    #[test]
    fn from_tty_false() {
        let info = StreamInfo::from_tty(false);
        assert!(!info.tty);
        assert!(info.piped);
    }

    #[test]
    fn stream_info_serialization() {
        let info = StreamInfo {
            tty: true,
            piped: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"tty\":true"));
        assert!(json.contains("\"piped\":false"));
    }

    #[test]
    fn stream_info_deserialization() {
        let json = r#"{"tty":false,"piped":true}"#;
        let info: StreamInfo = serde_json::from_str(json).unwrap();
        assert!(!info.tty);
        assert!(info.piped);
    }

    #[test]
    fn stream_info_consistency() {
        // Ensure tty and piped are always opposite
        let info = StreamInfo::from_tty(true);
        assert!(info.tty);
        assert!(!info.piped);

        let info = StreamInfo::from_tty(false);
        assert!(!info.tty);
        assert!(info.piped);
    }

    #[test]
    fn stream_info_manual_construction() {
        let info = StreamInfo {
            tty: true,
            piped: false,
        };
        assert!(info.tty);
        assert!(!info.piped);
    }

    #[test]
    fn stream_info_serialization_edge_cases() {
        let info = StreamInfo {
            tty: false,
            piped: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"tty\":false"));
        assert!(json.contains("\"piped\":true"));
    }

    #[test]
    fn stream_info_deserialization_edge_cases() {
        let json = r#"{"tty":true,"piped":false}"#;
        let info: StreamInfo = serde_json::from_str(json).unwrap();
        assert!(info.tty);
        assert!(!info.piped);
    }

    #[test]
    fn stream_info_deserialization_extra_fields() {
        let json = r#"{"tty":true,"piped":false,"unknown":"ignored"}"#;
        let info: StreamInfo = serde_json::from_str(json).unwrap();
        assert!(info.tty);
        assert!(!info.piped);
        // Unknown fields should be ignored
    }
}
