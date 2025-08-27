use crate::detectors::confidence::{HIGH, LOW, MEDIUM};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Declarative mapping for environment variable detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvMapping {
    /// The identifier this mapping produces (e.g., "replit", "cursor")
    pub id: String,
    /// The confidence level when this mapping matches
    pub confidence: f32,
    /// Environment variable patterns that indicate this environment
    pub indicators: Vec<EnvIndicator>,
    /// Additional facets to set when this mapping matches
    #[serde(default)]
    pub facets: HashMap<String, String>,
    /// Contexts to add when this mapping matches
    #[serde(default)]
    pub contexts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvIndicator {
    /// Environment variable name
    pub key: String,
    /// Expected value (if None, just check for presence)
    #[serde(default)]
    pub value: Option<String>,
    /// Whether this is a required indicator (all required must match)
    #[serde(default)]
    pub required: bool,
    /// Whether to check if the key starts with this prefix
    #[serde(default)]
    pub prefix: bool,
}

impl EnvMapping {
    /// Check if this mapping matches the given environment variables
    pub fn matches(&self, env_vars: &HashMap<String, String>) -> bool {
        let mut required_indicators = Vec::new();
        let mut optional_indicators = Vec::new();

        // Separate required and optional indicators
        for indicator in &self.indicators {
            if indicator.required {
                required_indicators.push(indicator);
            } else {
                optional_indicators.push(indicator);
            }
        }

        // All required indicators must match
        for indicator in &required_indicators {
            if !self.indicator_matches(indicator, env_vars) {
                return false;
            }
        }

        // At least one optional indicator must match (if there are any)
        if !optional_indicators.is_empty() {
            let any_optional_matches = optional_indicators
                .iter()
                .any(|indicator| self.indicator_matches(indicator, env_vars));
            if !any_optional_matches {
                return false;
            }
        }

        true
    }

    fn indicator_matches(
        &self,
        indicator: &EnvIndicator,
        env_vars: &HashMap<String, String>,
    ) -> bool {
        if indicator.prefix {
            // Check if any key starts with the prefix
            env_vars.keys().any(|key| key.starts_with(&indicator.key))
        } else {
            // Check exact key match
            match env_vars.get(&indicator.key) {
                Some(value) => {
                    // If we expect a specific value, check it
                    if let Some(expected_value) = &indicator.value {
                        value == expected_value
                    } else {
                        // Just check for presence
                        true
                    }
                }
                None => false,
            }
        }
    }

    /// Get the evidence key-value pairs that support this detection
    pub fn get_evidence(
        &self,
        env_vars: &HashMap<String, String>,
    ) -> Vec<(String, Option<String>)> {
        let mut evidence = Vec::new();

        for indicator in &self.indicators {
            if indicator.prefix {
                // For prefix matches, collect all matching keys
                for (key, value) in env_vars {
                    if key.starts_with(&indicator.key) {
                        evidence.push((key.clone(), Some(value.clone())));
                    }
                }
            } else if let Some(value) = env_vars.get(&indicator.key) {
                evidence.push((indicator.key.clone(), Some(value.clone())));
            }
        }

        evidence
    }
}

/// Predefined environment mappings for common environments
pub fn get_agent_mappings() -> Vec<EnvMapping> {
    vec![
        // Replit detection
        EnvMapping {
            id: "replit-agent".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "REPL_ID".to_string(),
                value: None,
                required: false,
                prefix: false,
            }],
            facets: HashMap::from([("host".to_string(), "replit".to_string())]),
            contexts: vec!["agent".to_string()],
        },
        // Cursor detection
        EnvMapping {
            id: "cursor".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "CURSOR_AGENT".to_string(),
                value: None,
                required: false,
                prefix: false,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
        },
        // Claude Code detection
        EnvMapping {
            id: "claude-code".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "CLAUDECODE".to_string(),
                value: None,
                required: false,
                prefix: false,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
        },
        // Cline detection
        EnvMapping {
            id: "cline".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "CLINE_ACTIVE".to_string(),
                value: None,
                required: false,
                prefix: false,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
        },
        // OpenHands detection
        EnvMapping {
            id: "openhands".to_string(),
            confidence: MEDIUM,
            indicators: vec![EnvIndicator {
                key: "SANDBOX_".to_string(),
                value: None,
                required: false,
                prefix: true,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
        },
        // Aider detection
        EnvMapping {
            id: "aider".to_string(),
            confidence: MEDIUM,
            indicators: vec![EnvIndicator {
                key: "AIDER_".to_string(),
                value: None,
                required: false,
                prefix: true,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
        },
        // Generic code agent detection
        EnvMapping {
            id: "unknown".to_string(),
            confidence: LOW,
            indicators: vec![EnvIndicator {
                key: "IS_CODE_AGENT".to_string(),
                value: Some("1".to_string()),
                required: false,
                prefix: false,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
        },
    ]
}

pub fn get_host_mappings() -> Vec<EnvMapping> {
    vec![
        // Replit host detection
        EnvMapping {
            id: "replit-host".to_string(),
            confidence: LOW,
            indicators: vec![EnvIndicator {
                key: "REPLIT_".to_string(),
                value: None,
                required: false,
                prefix: true,
            }],
            facets: HashMap::from([("host".to_string(), "replit".to_string())]),
            contexts: vec![],
        },
        // Codespaces detection
        EnvMapping {
            id: "codespaces".to_string(),
            confidence: LOW,
            indicators: vec![EnvIndicator {
                key: "GITHUB_CODESPACES_".to_string(),
                value: None,
                required: false,
                prefix: true,
            }],
            facets: HashMap::from([("host".to_string(), "codespaces".to_string())]),
            contexts: vec![],
        },
        // CI detection
        EnvMapping {
            id: "ci".to_string(),
            confidence: LOW,
            indicators: vec![
                EnvIndicator {
                    key: "GITHUB_ACTIONS".to_string(),
                    value: Some("1".to_string()),
                    required: false,
                    prefix: false,
                },
                EnvIndicator {
                    key: "CI".to_string(),
                    value: Some("1".to_string()),
                    required: false,
                    prefix: false,
                },
            ],
            facets: HashMap::from([("host".to_string(), "ci".to_string())]),
            contexts: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replit_agent_mapping() {
        let mappings = get_agent_mappings();
        let replit_mapping = mappings.iter().find(|m| m.id == "replit-agent").unwrap();

        let env_vars = HashMap::from([("REPL_ID".to_string(), "abc123".to_string())]);

        assert!(replit_mapping.matches(&env_vars));
        assert_eq!(replit_mapping.confidence, HIGH);
    }

    #[test]
    fn test_cursor_mapping() {
        let mappings = get_agent_mappings();
        let cursor_mapping = mappings.iter().find(|m| m.id == "cursor").unwrap();

        let env_vars = HashMap::from([("CURSOR_AGENT".to_string(), "1".to_string())]);

        assert!(cursor_mapping.matches(&env_vars));
        assert_eq!(cursor_mapping.confidence, HIGH);
    }

    #[test]
    fn test_openhands_prefix_mapping() {
        let mappings = get_agent_mappings();
        let openhands_mapping = mappings.iter().find(|m| m.id == "openhands").unwrap();

        let env_vars = HashMap::from([
            ("SANDBOX_VOLUMES".to_string(), "/tmp".to_string()),
            (
                "SANDBOX_RUNTIME_CONTAINER_IMAGE".to_string(),
                "alpine".to_string(),
            ),
        ]);

        assert!(openhands_mapping.matches(&env_vars));
    }

    #[test]
    fn test_aider_mapping() {
        let mappings = get_agent_mappings();
        let aider_mapping = mappings.iter().find(|m| m.id == "aider").unwrap();

        let env_vars = HashMap::from([("AIDER_MODEL".to_string(), "gpt-4o-mini".to_string())]);

        assert!(aider_mapping.matches(&env_vars));
    }
}
