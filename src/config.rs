use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliConfig {
    #[serde(default)]
    pub error_handling: ErrorHandlingConfig,
    #[serde(default)]
    pub output_formatting: OutputFormattingConfig,
    #[serde(default)]
    pub validation: ValidationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ErrorHandlingConfig {
    pub strict_mode: bool,
    pub show_usage_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputFormattingConfig {
    pub context_descriptions: bool,
    pub nested_display: bool,
    pub rainbow_colors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ValidationConfig {
    pub validate_predicates: bool,
    pub allowed_characters: String,
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            show_usage_on_error: true,
        }
    }
}

impl Default for OutputFormattingConfig {
    fn default() -> Self {
        Self {
            context_descriptions: true,
            nested_display: true,
            rainbow_colors: true,
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            validate_predicates: true,
            allowed_characters: "a-zA-Z0-9_.=-".to_string(),
        }
    }
}

impl CliConfig {
    pub fn load() -> Self {
        // Try to load from config file, fallback to default
        if let Some(config_path) = Self::config_file_path()
            && let Ok(content) = std::fs::read_to_string(config_path)
            && let Ok(config) = toml::from_str(&content)
        {
            return config;
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_path) = Self::config_file_path() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let content = toml::to_string_pretty(self)?;
            std::fs::write(config_path, content)?;
        }
        Ok(())
    }

    fn config_file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("envsense");
            path.push("config.toml");
            path
        })
    }

    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("envsense");
            path
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert!(config.error_handling.strict_mode);
        assert!(config.error_handling.show_usage_on_error);
        assert!(config.output_formatting.context_descriptions);
        assert!(config.output_formatting.nested_display);
        assert!(config.output_formatting.rainbow_colors);
        assert!(config.validation.validate_predicates);
        assert_eq!(config.validation.allowed_characters, "a-zA-Z0-9_.=-");
    }

    #[test]
    fn test_config_serialization() {
        let config = CliConfig::default();
        let toml_str = toml::to_string(&config).unwrap();

        // Should contain all sections
        assert!(toml_str.contains("[error_handling]"));
        assert!(toml_str.contains("[output_formatting]"));
        assert!(toml_str.contains("[validation]"));

        // Should contain expected values
        assert!(toml_str.contains("strict_mode = true"));
        assert!(toml_str.contains("context_descriptions = true"));
        assert!(toml_str.contains("validate_predicates = true"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[error_handling]
strict_mode = false
show_usage_on_error = false

[output_formatting]
context_descriptions = false
nested_display = false
rainbow_colors = false

[validation]
validate_predicates = false
allowed_characters = "a-z"
"#;

        let config: CliConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.error_handling.strict_mode);
        assert!(!config.error_handling.show_usage_on_error);
        assert!(!config.output_formatting.context_descriptions);
        assert!(!config.output_formatting.nested_display);
        assert!(!config.output_formatting.rainbow_colors);
        assert!(!config.validation.validate_predicates);
        assert_eq!(config.validation.allowed_characters, "a-z");
    }

    #[test]
    fn test_config_partial_deserialization() {
        // Test that partial config files work with defaults for missing sections
        let toml_str = r#"
[error_handling]
strict_mode = false
"#;

        let config: CliConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.error_handling.strict_mode);
        // These should use defaults since they're missing from the TOML
        assert!(config.error_handling.show_usage_on_error);
        assert!(config.output_formatting.context_descriptions);
        assert!(config.validation.validate_predicates);
    }
}
