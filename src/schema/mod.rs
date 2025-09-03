pub mod evidence;
pub mod main;
pub mod nested;

// Re-export commonly used types
pub use evidence::{Evidence, Signal};
pub use main::EnvSense;
pub use nested::NewEnvSense;

// Schema version constants
pub const SCHEMA_VERSION: &str = "0.3.0"; // Current schema version

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_constants() {
        assert_eq!(SCHEMA_VERSION, "0.3.0");
    }

    #[test]
    fn new_envsense_default() {
        let new = EnvSense::default();
        assert_eq!(new.version, SCHEMA_VERSION);
        assert!(new.contexts.is_empty());
        assert_eq!(new.traits.agent.id, None);
        assert_eq!(new.traits.ide.id, None);
        assert_eq!(new.traits.ci.id, None);
    }

    #[test]
    fn empty_contexts_handling() {
        let new = EnvSense::default();
        assert!(new.contexts.is_empty());

        let json = serde_json::to_string(&new).unwrap();
        assert!(json.contains("\"contexts\":[]"));
    }

    #[test]
    fn nested_traits_serialization() {
        let new = EnvSense::default();
        let json = serde_json::to_string(&new.traits).unwrap();
        assert!(json.contains("\"agent\":"));
        assert!(json.contains("\"terminal\":"));
        assert!(json.contains("\"ide\":"));
        assert!(json.contains("\"ci\":"));
    }
}
