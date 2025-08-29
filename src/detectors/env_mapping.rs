use crate::detectors::confidence::{HIGH, LOW, MEDIUM};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
    /// Value mappings specific to this environment (only applied when this mapping matches)
    #[serde(default)]
    pub value_mappings: Vec<ValueMapping>,
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
    /// Whether to check if the value contains this substring (case-insensitive)
    #[serde(default)]
    pub contains: Option<String>,
    /// Priority for ordering matches (higher number = higher priority)
    #[serde(default)]
    pub priority: u8,
}

/// Condition for conditional value mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    /// Check if a previously extracted value equals a specific value
    Equals(String, serde_json::Value),
    /// Check if a previously extracted value is not equal to a specific value
    NotEquals(String, serde_json::Value),
    /// Check if a previously extracted value contains a substring
    Contains(String, String),
    /// Check if a previously extracted value is truthy (non-empty, non-false, non-zero)
    IsTruthy(String),
    /// Check if a previously extracted value is falsy (empty, false, zero)
    IsFalsy(String),
    /// Check if a previously extracted value exists
    Exists(String),
    /// Check if a previously extracted value does not exist
    NotExists(String),
}

impl Condition {
    /// Evaluate the condition against previously extracted values
    pub fn evaluate(&self, extracted_values: &HashMap<String, serde_json::Value>) -> bool {
        match self {
            Condition::Equals(key, expected_value) => {
                extracted_values.get(key) == Some(expected_value)
            }
            Condition::NotEquals(key, expected_value) => {
                extracted_values.get(key) != Some(expected_value)
            }
            Condition::Contains(key, substring) => extracted_values.get(key).is_some_and(|value| {
                value
                    .as_str()
                    .is_some_and(|s| s.to_lowercase().contains(&substring.to_lowercase()))
            }),
            Condition::IsTruthy(key) => {
                extracted_values.get(key).is_some_and(|value| match value {
                    serde_json::Value::String(s) => !s.is_empty(),
                    serde_json::Value::Bool(b) => *b,
                    serde_json::Value::Number(n) => n.as_i64().is_some_and(|i| i != 0),
                    _ => false,
                })
            }
            Condition::IsFalsy(key) => extracted_values.get(key).is_none_or(|value| match value {
                serde_json::Value::String(s) => s.is_empty(),
                serde_json::Value::Bool(b) => !*b,
                serde_json::Value::Number(n) => n.as_i64().is_none_or(|i| i == 0),
                _ => true,
            }),
            Condition::Exists(key) => extracted_values.contains_key(key),
            Condition::NotExists(key) => !extracted_values.contains_key(key),
        }
    }
}

/// Value mapping for extracting specific values from environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueMapping {
    /// The key this value will be stored under in the result
    pub target_key: String,
    /// The environment variable to extract the value from
    pub source_key: String,
    /// Whether this value extraction is required
    #[serde(default)]
    pub required: bool,
    /// Transformation to apply to the value
    #[serde(default)]
    pub transform: Option<ValueTransform>,
    /// Condition that must be met for this mapping to be applied
    #[serde(default)]
    pub condition: Option<Condition>,
}

/// Value transformation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueTransform {
    /// Convert to boolean (non-empty = true, empty = false)
    ToBool,
    /// Convert to lowercase
    ToLowercase,
    /// Check if equals specific value, return boolean
    Equals(String),
    /// Check if contains substring, return boolean
    Contains(String),
    /// Parse as integer
    ToInt,
    /// Convert to uppercase
    ToUppercase,
    /// Trim whitespace
    Trim,
    /// Replace substring
    Replace { from: String, to: String },
    /// Split string and get specific index
    Split { delimiter: String, index: usize },
    /// Custom transformation function
    Custom(String),
}

impl ValueTransform {
    /// Apply the transformation to a value
    pub fn apply(&self, value: &str) -> Result<serde_json::Value, String> {
        match self {
            ValueTransform::ToBool => Ok(json!(!value.is_empty())),
            ValueTransform::ToLowercase => Ok(json!(value.to_lowercase())),
            ValueTransform::Equals(target) => Ok(json!(value == target)),
            ValueTransform::Contains(substring) => Ok(json!(
                value.to_lowercase().contains(&substring.to_lowercase())
            )),
            ValueTransform::ToInt => value
                .parse::<i64>()
                .map(|i| json!(i))
                .map_err(|e| format!("Failed to parse '{}' as integer: {}", value, e)),
            ValueTransform::ToUppercase => Ok(json!(value.to_uppercase())),
            ValueTransform::Trim => Ok(json!(value.trim())),
            ValueTransform::Replace { from, to } => Ok(json!(value.replace(from, to))),
            ValueTransform::Split { delimiter, index } => {
                let parts: Vec<&str> = value.split(delimiter).collect();
                if *index < parts.len() {
                    Ok(json!(parts[*index]))
                } else {
                    Err(format!(
                        "Split index {} out of bounds for value '{}'",
                        index, value
                    ))
                }
            }
            ValueTransform::Custom(func_name) => {
                // Future: plugin system for custom transformations
                Err(format!(
                    "Custom transformation '{}' not implemented",
                    func_name
                ))
            }
        }
    }
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
                    if let Some(expected_value) = &indicator.value
                        && value != expected_value
                    {
                        return false;
                    }

                    // If we expect the value to contain a substring, check it
                    if let Some(contains_value) = &indicator.contains
                        && !value
                            .to_lowercase()
                            .contains(&contains_value.to_lowercase())
                    {
                        return false;
                    }

                    // All checks passed
                    true
                }
                None => false,
            }
        }
    }

    /// Get the highest priority indicator for this mapping
    pub fn get_highest_priority(&self) -> u8 {
        self.indicators
            .iter()
            .map(|i| i.priority)
            .max()
            .unwrap_or(0)
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

    /// Extract values from environment variables according to value mappings
    pub fn extract_values(
        &self,
        env_vars: &HashMap<String, String>,
    ) -> HashMap<String, serde_json::Value> {
        let mut extracted = HashMap::new();

        // Process mappings in dependency order (no conditions first, then conditional ones)
        let mappings_to_process: Vec<&ValueMapping> = self.value_mappings.iter().collect();
        let mut processed_count = 0;

        while processed_count < mappings_to_process.len() {
            let initial_count = processed_count;

            for mapping in &mappings_to_process {
                // Skip if already processed
                if extracted.contains_key(&mapping.target_key) {
                    continue;
                }

                // Check if condition is met (if any)
                if let Some(condition) = &mapping.condition
                    && !condition.evaluate(&extracted)
                {
                    continue; // Skip this mapping if condition not met
                }

                // Process the mapping
                if let Some(value) = env_vars.get(&mapping.source_key) {
                    match mapping.transform.as_ref() {
                        Some(transform) => {
                            match transform.apply(value) {
                                Ok(transformed) => {
                                    extracted.insert(mapping.target_key.clone(), transformed);
                                    processed_count += 1;
                                }
                                Err(e) => {
                                    // Log error but continue with other mappings
                                    eprintln!(
                                        "Warning: Failed to transform {}: {}",
                                        mapping.source_key, e
                                    );
                                }
                            }
                        }
                        None => {
                            extracted.insert(mapping.target_key.clone(), json!(value));
                            processed_count += 1;
                        }
                    }
                } else if mapping.required {
                    eprintln!(
                        "Warning: Required value mapping missing: {}",
                        mapping.source_key
                    );
                }
            }

            // If no new mappings were processed in this iteration, we're done
            if processed_count == initial_count {
                break;
            }
        }

        extracted
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
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("host".to_string(), "replit".to_string())]),
            contexts: vec!["agent".to_string()],
            value_mappings: vec![],
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
                contains: None,
                priority: 0,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
            value_mappings: vec![],
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
                contains: None,
                priority: 0,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
            value_mappings: vec![],
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
                contains: None,
                priority: 0,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
            value_mappings: vec![],
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
                contains: None,
                priority: 0,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
            value_mappings: vec![],
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
                contains: None,
                priority: 0,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
            value_mappings: vec![],
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
                contains: None,
                priority: 0,
            }],
            facets: HashMap::new(),
            contexts: vec!["agent".to_string()],
            value_mappings: vec![],
        },
    ]
}

/// Predefined environment mappings for IDE detection
pub fn get_ide_mappings() -> Vec<EnvMapping> {
    vec![
        // Cursor IDE detection (highest priority)
        EnvMapping {
            id: "cursor-ide".to_string(),
            confidence: HIGH,
            indicators: vec![
                EnvIndicator {
                    key: "TERM_PROGRAM".to_string(),
                    value: Some("vscode".to_string()),
                    required: true,
                    prefix: false,
                    contains: None,
                    priority: 3, // Highest priority
                },
                EnvIndicator {
                    key: "CURSOR_TRACE_ID".to_string(),
                    value: None,
                    required: true,
                    prefix: false,
                    contains: None,
                    priority: 3,
                },
            ],
            facets: HashMap::from([("ide_id".to_string(), "cursor".to_string())]),
            contexts: vec!["ide".to_string()],
            value_mappings: vec![],
        },
        // VS Code Insiders detection (medium priority)
        EnvMapping {
            id: "vscode-insiders".to_string(),
            confidence: HIGH,
            indicators: vec![
                EnvIndicator {
                    key: "TERM_PROGRAM".to_string(),
                    value: Some("vscode".to_string()),
                    required: true,
                    prefix: false,
                    contains: None,
                    priority: 2,
                },
                EnvIndicator {
                    key: "TERM_PROGRAM_VERSION".to_string(),
                    value: None,
                    required: true,
                    prefix: false,
                    contains: Some("insider".to_string()),
                    priority: 2,
                },
            ],
            facets: HashMap::from([("ide_id".to_string(), "vscode-insiders".to_string())]),
            contexts: vec!["ide".to_string()],
            value_mappings: vec![],
        },
        // VS Code detection (lowest priority)
        EnvMapping {
            id: "vscode".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "TERM_PROGRAM".to_string(),
                value: Some("vscode".to_string()),
                required: true,
                prefix: false,
                contains: None,
                priority: 1,
            }],
            facets: HashMap::from([("ide_id".to_string(), "vscode".to_string())]),
            contexts: vec!["ide".to_string()],
            value_mappings: vec![],
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
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("host".to_string(), "replit".to_string())]),
            contexts: vec![],
            value_mappings: vec![],
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
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("host".to_string(), "codespaces".to_string())]),
            contexts: vec![],
            value_mappings: vec![],
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
                    contains: None,
                    priority: 0,
                },
                EnvIndicator {
                    key: "CI".to_string(),
                    value: Some("1".to_string()),
                    required: false,
                    prefix: false,
                    contains: None,
                    priority: 0,
                },
            ],
            facets: HashMap::from([("host".to_string(), "ci".to_string())]),
            contexts: vec![],
            value_mappings: vec![],
        },
        // GitHub Actions detection
        EnvMapping {
            id: "github-actions".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "GITHUB_ACTIONS".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "github_actions".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // GitLab CI detection
        EnvMapping {
            id: "gitlab-ci".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "GITLAB_CI".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "gitlab_ci".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // CircleCI detection
        EnvMapping {
            id: "circleci".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "CIRCLECI".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "circleci".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // Buildkite detection
        EnvMapping {
            id: "buildkite".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "BUILDKITE".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "buildkite".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // Jenkins detection
        EnvMapping {
            id: "jenkins".to_string(),
            confidence: HIGH,
            indicators: vec![
                EnvIndicator {
                    key: "JENKINS_URL".to_string(),
                    value: None,
                    required: false,
                    prefix: false,
                    contains: None,
                    priority: 0,
                },
                EnvIndicator {
                    key: "JENKINS_HOME".to_string(),
                    value: None,
                    required: false,
                    prefix: false,
                    contains: None,
                    priority: 0,
                },
            ],
            facets: HashMap::from([("ci_id".to_string(), "jenkins".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // TeamCity detection
        EnvMapping {
            id: "teamcity".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "TEAMCITY_VERSION".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "teamcity".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // Bitbucket Pipelines detection
        EnvMapping {
            id: "bitbucket-pipelines".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "BITBUCKET_BUILD_NUMBER".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "bitbucket_pipelines".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // Azure Pipelines detection
        EnvMapping {
            id: "azure-pipelines".to_string(),
            confidence: HIGH,
            indicators: vec![
                EnvIndicator {
                    key: "AZURE_HTTP_USER_AGENT".to_string(),
                    value: None,
                    required: false,
                    prefix: false,
                    contains: None,
                    priority: 0,
                },
                EnvIndicator {
                    key: "TF_BUILD".to_string(),
                    value: None,
                    required: false,
                    prefix: false,
                    contains: None,
                    priority: 0,
                },
            ],
            facets: HashMap::from([("ci_id".to_string(), "azure_pipelines".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // Google Cloud Build detection
        EnvMapping {
            id: "google-cloud-build".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "GOOGLE_CLOUD_BUILD".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "google_cloud_build".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // Vercel detection
        EnvMapping {
            id: "vercel".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "VERCEL".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "vercel".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // AWS CodeBuild detection
        EnvMapping {
            id: "aws-codebuild".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "CODEBUILD_BUILD_ID".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "aws_codebuild".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // SourceHut detection
        EnvMapping {
            id: "sourcehut".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "BUILD_REASON".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "sourcehut".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // AppVeyor detection
        EnvMapping {
            id: "appveyor".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "APPVEYOR".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "appveyor".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
    ]
}

/// Predefined environment mappings for CI detection
pub fn get_ci_mappings() -> Vec<EnvMapping> {
    vec![
        // GitHub Actions detection
        EnvMapping {
            id: "github-actions".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "GITHUB_ACTIONS".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "github_actions".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "GITHUB_REF_NAME".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                },
                ValueMapping {
                    target_key: "is_pr".to_string(),
                    source_key: "GITHUB_EVENT_NAME".to_string(),
                    required: false,
                    transform: Some(ValueTransform::Equals("pull_request".to_string())),
                    condition: None,
                },
                ValueMapping {
                    target_key: "pr_number".to_string(),
                    source_key: "GITHUB_EVENT_NUMBER".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToInt),
                    condition: None,
                },
                ValueMapping {
                    target_key: "repository".to_string(),
                    source_key: "GITHUB_REPOSITORY".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                },
                ValueMapping {
                    target_key: "workflow".to_string(),
                    source_key: "GITHUB_WORKFLOW".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                },
            ],
        },
        // GitLab CI detection
        EnvMapping {
            id: "gitlab-ci".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "GITLAB_CI".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "gitlab_ci".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "CI_COMMIT_REF_NAME".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                },
                ValueMapping {
                    target_key: "is_pr".to_string(),
                    source_key: "CI_MERGE_REQUEST_ID".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToBool),
                    condition: None,
                },
                ValueMapping {
                    target_key: "pipeline_id".to_string(),
                    source_key: "CI_PIPELINE_ID".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToInt),
                    condition: None,
                },
                ValueMapping {
                    target_key: "project_path".to_string(),
                    source_key: "CI_PROJECT_PATH".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                },
            ],
        },
        // CircleCI detection
        EnvMapping {
            id: "circleci".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "CIRCLECI".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "circleci".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "CIRCLE_BRANCH".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                },
                ValueMapping {
                    target_key: "is_pr".to_string(),
                    source_key: "CIRCLE_PR_NUMBER".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToBool),
                    condition: None,
                },
                ValueMapping {
                    target_key: "build_number".to_string(),
                    source_key: "CIRCLE_BUILD_NUM".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToInt),
                    condition: None,
                },
                ValueMapping {
                    target_key: "project_name".to_string(),
                    source_key: "CIRCLE_PROJECT_REPONAME".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                },
            ],
        },
        // Buildkite detection
        EnvMapping {
            id: "buildkite".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "BUILDKITE".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "buildkite".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // Jenkins detection
        EnvMapping {
            id: "jenkins".to_string(),
            confidence: HIGH,
            indicators: vec![
                EnvIndicator {
                    key: "JENKINS_URL".to_string(),
                    value: None,
                    required: false,
                    prefix: false,
                    contains: None,
                    priority: 0,
                },
                EnvIndicator {
                    key: "JENKINS_HOME".to_string(),
                    value: None,
                    required: false,
                    prefix: false,
                    contains: None,
                    priority: 0,
                },
            ],
            facets: HashMap::from([("ci_id".to_string(), "jenkins".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // TeamCity detection
        EnvMapping {
            id: "teamcity".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "TEAMCITY_VERSION".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "teamcity".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // Bitbucket Pipelines detection
        EnvMapping {
            id: "bitbucket-pipelines".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "BITBUCKET_BUILD_NUMBER".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "bitbucket_pipelines".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // Azure Pipelines detection
        EnvMapping {
            id: "azure-pipelines".to_string(),
            confidence: HIGH,
            indicators: vec![
                EnvIndicator {
                    key: "AZURE_HTTP_USER_AGENT".to_string(),
                    value: None,
                    required: false,
                    prefix: false,
                    contains: None,
                    priority: 0,
                },
                EnvIndicator {
                    key: "TF_BUILD".to_string(),
                    value: None,
                    required: false,
                    prefix: false,
                    contains: None,
                    priority: 0,
                },
            ],
            facets: HashMap::from([("ci_id".to_string(), "azure_pipelines".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // Google Cloud Build detection
        EnvMapping {
            id: "google-cloud-build".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "GOOGLE_CLOUD_BUILD".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "google_cloud_build".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // Vercel detection
        EnvMapping {
            id: "vercel".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "VERCEL".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "vercel".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // AWS CodeBuild detection
        EnvMapping {
            id: "aws-codebuild".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "CODEBUILD_BUILD_ID".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "aws_codebuild".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // SourceHut detection
        EnvMapping {
            id: "sourcehut".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "BUILD_REASON".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "sourcehut".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
        },
        // AppVeyor detection
        EnvMapping {
            id: "appveyor".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "APPVEYOR".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "appveyor".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![],
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

    #[test]
    fn test_value_transform_to_bool() {
        let transform = ValueTransform::ToBool;

        assert_eq!(transform.apply("").unwrap(), json!(false));
        assert_eq!(transform.apply("value").unwrap(), json!(true));
        assert_eq!(transform.apply("123").unwrap(), json!(true));
    }

    #[test]
    fn test_value_transform_equals() {
        let transform = ValueTransform::Equals("pull_request".to_string());

        assert_eq!(transform.apply("pull_request").unwrap(), json!(true));
        assert_eq!(transform.apply("push").unwrap(), json!(false));
        assert_eq!(transform.apply("PULL_REQUEST").unwrap(), json!(false)); // Case sensitive
    }

    #[test]
    fn test_value_transform_contains() {
        let transform = ValueTransform::Contains("true".to_string());

        assert_eq!(transform.apply("true").unwrap(), json!(true));
        assert_eq!(transform.apply("TRUE").unwrap(), json!(true)); // Case insensitive
        assert_eq!(transform.apply("is_true").unwrap(), json!(true));
        assert_eq!(transform.apply("false").unwrap(), json!(false));
    }

    #[test]
    fn test_value_transform_to_int() {
        let transform = ValueTransform::ToInt;

        assert_eq!(transform.apply("123").unwrap(), json!(123));
        assert_eq!(transform.apply("-456").unwrap(), json!(-456));
        assert!(transform.apply("not_a_number").is_err());
    }

    #[test]
    fn test_value_transform_to_uppercase() {
        let transform = ValueTransform::ToUppercase;

        assert_eq!(transform.apply("hello").unwrap(), json!("HELLO"));
        assert_eq!(transform.apply("World").unwrap(), json!("WORLD"));
        assert_eq!(transform.apply("123").unwrap(), json!("123"));
    }

    #[test]
    fn test_value_transform_trim() {
        let transform = ValueTransform::Trim;

        assert_eq!(transform.apply("  hello  ").unwrap(), json!("hello"));
        assert_eq!(transform.apply("world\n").unwrap(), json!("world"));
        assert_eq!(transform.apply("  ").unwrap(), json!(""));
    }

    #[test]
    fn test_value_transform_replace() {
        let transform = ValueTransform::Replace {
            from: "old".to_string(),
            to: "new".to_string(),
        };

        assert_eq!(transform.apply("old_value").unwrap(), json!("new_value"));
        assert_eq!(
            transform.apply("no_old_here").unwrap(),
            json!("no_new_here")
        );
        assert_eq!(transform.apply("").unwrap(), json!(""));
    }

    #[test]
    fn test_value_transform_split() {
        let transform = ValueTransform::Split {
            delimiter: "/".to_string(),
            index: 1,
        };

        assert_eq!(transform.apply("a/b/c").unwrap(), json!("b"));
        assert_eq!(transform.apply("owner/repo").unwrap(), json!("repo"));
        assert!(transform.apply("single").is_err()); // Index 1 out of bounds
        assert_eq!(transform.apply("a/b").unwrap(), json!("b")); // Index 1 exists for "a/b"
    }

    #[test]
    fn test_github_actions_value_extraction() {
        let mappings = get_ci_mappings();
        let github_mapping = mappings.iter().find(|m| m.id == "github-actions").unwrap();

        let env_vars = HashMap::from([
            ("GITHUB_ACTIONS".to_string(), "true".to_string()),
            ("GITHUB_REF_NAME".to_string(), "main".to_string()),
            ("GITHUB_EVENT_NAME".to_string(), "pull_request".to_string()),
            ("GITHUB_EVENT_NUMBER".to_string(), "42".to_string()),
            ("GITHUB_REPOSITORY".to_string(), "owner/repo".to_string()),
            ("GITHUB_WORKFLOW".to_string(), "CI".to_string()),
        ]);

        // Test that the mapping matches
        assert!(github_mapping.matches(&env_vars));

        // Test value extraction
        let extracted = github_mapping.extract_values(&env_vars);

        assert_eq!(extracted.get("branch").unwrap(), &json!("main"));
        assert_eq!(extracted.get("is_pr").unwrap(), &json!(true));
        assert_eq!(extracted.get("pr_number").unwrap(), &json!(42));
        assert_eq!(extracted.get("repository").unwrap(), &json!("owner/repo"));
        assert_eq!(extracted.get("workflow").unwrap(), &json!("CI"));
    }

    #[test]
    fn test_gitlab_ci_value_extraction() {
        let mappings = get_ci_mappings();
        let gitlab_mapping = mappings.iter().find(|m| m.id == "gitlab-ci").unwrap();

        let env_vars = HashMap::from([
            ("GITLAB_CI".to_string(), "true".to_string()),
            (
                "CI_COMMIT_REF_NAME".to_string(),
                "feature-branch".to_string(),
            ),
            ("CI_MERGE_REQUEST_ID".to_string(), "123".to_string()),
            ("CI_PIPELINE_ID".to_string(), "456".to_string()),
            ("CI_PROJECT_PATH".to_string(), "group/project".to_string()),
        ]);

        // Test that the mapping matches
        assert!(gitlab_mapping.matches(&env_vars));

        // Test value extraction
        let extracted = gitlab_mapping.extract_values(&env_vars);

        assert_eq!(extracted.get("branch").unwrap(), &json!("feature-branch"));
        assert_eq!(extracted.get("is_pr").unwrap(), &json!(true)); // Non-empty string = true
        assert_eq!(extracted.get("pipeline_id").unwrap(), &json!(456));
        assert_eq!(
            extracted.get("project_path").unwrap(),
            &json!("group/project")
        );
    }

    #[test]
    fn test_circleci_value_extraction() {
        let mappings = get_ci_mappings();
        let circle_mapping = mappings.iter().find(|m| m.id == "circleci").unwrap();

        let env_vars = HashMap::from([
            ("CIRCLECI".to_string(), "true".to_string()),
            ("CIRCLE_BRANCH".to_string(), "develop".to_string()),
            ("CIRCLE_PR_NUMBER".to_string(), "789".to_string()),
            ("CIRCLE_BUILD_NUM".to_string(), "1001".to_string()),
            (
                "CIRCLE_PROJECT_REPONAME".to_string(),
                "my-project".to_string(),
            ),
        ]);

        // Test that the mapping matches
        assert!(circle_mapping.matches(&env_vars));

        // Test value extraction
        let extracted = circle_mapping.extract_values(&env_vars);

        assert_eq!(extracted.get("branch").unwrap(), &json!("develop"));
        assert_eq!(extracted.get("is_pr").unwrap(), &json!(true)); // Non-empty string = true
        assert_eq!(extracted.get("build_number").unwrap(), &json!(1001));
        assert_eq!(extracted.get("project_name").unwrap(), &json!("my-project"));
    }

    #[test]
    fn test_condition_equals() {
        let mut extracted = HashMap::new();
        extracted.insert("is_pr".to_string(), json!(true));
        extracted.insert("branch".to_string(), json!("main"));

        let condition = Condition::Equals("is_pr".to_string(), json!(true));
        assert!(condition.evaluate(&extracted));

        let condition = Condition::Equals("is_pr".to_string(), json!(false));
        assert!(!condition.evaluate(&extracted));

        let condition = Condition::Equals("missing_key".to_string(), json!(true));
        assert!(!condition.evaluate(&extracted));
    }

    #[test]
    fn test_condition_not_equals() {
        let mut extracted = HashMap::new();
        extracted.insert("is_pr".to_string(), json!(true));

        let condition = Condition::NotEquals("is_pr".to_string(), json!(false));
        assert!(condition.evaluate(&extracted));

        let condition = Condition::NotEquals("is_pr".to_string(), json!(true));
        assert!(!condition.evaluate(&extracted));

        let condition = Condition::NotEquals("missing_key".to_string(), json!(true));
        assert!(condition.evaluate(&extracted)); // NotEquals returns true for missing keys
    }

    #[test]
    fn test_condition_contains() {
        let mut extracted = HashMap::new();
        extracted.insert("branch".to_string(), json!("feature/new-feature"));

        let condition = Condition::Contains("branch".to_string(), "feature".to_string());
        assert!(condition.evaluate(&extracted));

        let condition = Condition::Contains("branch".to_string(), "main".to_string());
        assert!(!condition.evaluate(&extracted));

        let condition = Condition::Contains("missing_key".to_string(), "feature".to_string());
        assert!(!condition.evaluate(&extracted));
    }

    #[test]
    fn test_condition_is_truthy() {
        let mut extracted = HashMap::new();
        extracted.insert("bool_true".to_string(), json!(true));
        extracted.insert("bool_false".to_string(), json!(false));
        extracted.insert("string_value".to_string(), json!("hello"));
        extracted.insert("empty_string".to_string(), json!(""));
        extracted.insert("number_positive".to_string(), json!(42));
        extracted.insert("number_zero".to_string(), json!(0));

        assert!(Condition::IsTruthy("bool_true".to_string()).evaluate(&extracted));
        assert!(!Condition::IsTruthy("bool_false".to_string()).evaluate(&extracted));
        assert!(Condition::IsTruthy("string_value".to_string()).evaluate(&extracted));
        assert!(!Condition::IsTruthy("empty_string".to_string()).evaluate(&extracted));
        assert!(Condition::IsTruthy("number_positive".to_string()).evaluate(&extracted));
        assert!(!Condition::IsTruthy("number_zero".to_string()).evaluate(&extracted));
        assert!(!Condition::IsTruthy("missing_key".to_string()).evaluate(&extracted));
    }

    #[test]
    fn test_condition_is_falsy() {
        let mut extracted = HashMap::new();
        extracted.insert("bool_true".to_string(), json!(true));
        extracted.insert("bool_false".to_string(), json!(false));
        extracted.insert("string_value".to_string(), json!("hello"));
        extracted.insert("empty_string".to_string(), json!(""));
        extracted.insert("number_positive".to_string(), json!(42));
        extracted.insert("number_zero".to_string(), json!(0));

        assert!(!Condition::IsFalsy("bool_true".to_string()).evaluate(&extracted));
        assert!(Condition::IsFalsy("bool_false".to_string()).evaluate(&extracted));
        assert!(!Condition::IsFalsy("string_value".to_string()).evaluate(&extracted));
        assert!(Condition::IsFalsy("empty_string".to_string()).evaluate(&extracted));
        assert!(!Condition::IsFalsy("number_positive".to_string()).evaluate(&extracted));
        assert!(Condition::IsFalsy("number_zero".to_string()).evaluate(&extracted));
        assert!(Condition::IsFalsy("missing_key".to_string()).evaluate(&extracted)); // Missing keys are falsy
    }

    #[test]
    fn test_condition_exists() {
        let mut extracted = HashMap::new();
        extracted.insert("exists".to_string(), json!("value"));

        assert!(Condition::Exists("exists".to_string()).evaluate(&extracted));
        assert!(!Condition::Exists("missing".to_string()).evaluate(&extracted));
    }

    #[test]
    fn test_condition_not_exists() {
        let mut extracted = HashMap::new();
        extracted.insert("exists".to_string(), json!("value"));

        assert!(!Condition::NotExists("exists".to_string()).evaluate(&extracted));
        assert!(Condition::NotExists("missing".to_string()).evaluate(&extracted));
    }

    #[test]
    fn test_conditional_value_mapping() {
        let mapping = EnvMapping {
            id: "test-conditional".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "TEST_ENV".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::new(),
            contexts: vec!["test".to_string()],
            value_mappings: vec![
                // First, extract is_pr
                ValueMapping {
                    target_key: "is_pr".to_string(),
                    source_key: "GITHUB_EVENT_NAME".to_string(),
                    required: false,
                    transform: Some(ValueTransform::Equals("pull_request".to_string())),
                    condition: None,
                },
                // Then, extract pr_number only if is_pr is true
                ValueMapping {
                    target_key: "pr_number".to_string(),
                    source_key: "GITHUB_EVENT_NUMBER".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToInt),
                    condition: Some(Condition::IsTruthy("is_pr".to_string())),
                },
                // Extract branch name regardless
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "GITHUB_REF_NAME".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                },
            ],
        };

        // Test with PR environment
        let pr_env = HashMap::from([
            ("GITHUB_EVENT_NAME".to_string(), "pull_request".to_string()),
            ("GITHUB_EVENT_NUMBER".to_string(), "42".to_string()),
            ("GITHUB_REF_NAME".to_string(), "feature-branch".to_string()),
        ]);

        let extracted = mapping.extract_values(&pr_env);
        assert_eq!(extracted.get("is_pr"), Some(&json!(true)));
        assert_eq!(extracted.get("pr_number"), Some(&json!(42)));
        assert_eq!(extracted.get("branch"), Some(&json!("feature-branch")));

        // Test with push environment (no PR)
        let push_env = HashMap::from([
            ("GITHUB_EVENT_NAME".to_string(), "push".to_string()),
            ("GITHUB_EVENT_NUMBER".to_string(), "42".to_string()),
            ("GITHUB_REF_NAME".to_string(), "main".to_string()),
        ]);

        let extracted = mapping.extract_values(&push_env);
        assert_eq!(extracted.get("is_pr"), Some(&json!(false)));
        assert_eq!(extracted.get("pr_number"), None); // Should not be extracted
        assert_eq!(extracted.get("branch"), Some(&json!("main")));
    }
}
