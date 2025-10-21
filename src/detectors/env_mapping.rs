use crate::detectors::confidence::{HIGH, LOW, MEDIUM};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// Validation error types for value mappings
#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    #[error("Missing required field: {field}")]
    MissingRequiredField { field: String },
    #[error("Invalid field value: {field} = {value} (expected: {expected})")]
    InvalidFieldValue {
        field: String,
        value: String,
        expected: String,
    },
    #[error("Invalid transformation: {transform}")]
    InvalidTransformation { transform: String },
    #[error("Invalid condition: {condition}")]
    InvalidCondition { condition: String },
    #[error("Circular dependency detected: {dependency_chain}")]
    CircularDependency { dependency_chain: String },
    #[error("Invalid target key format: {key}")]
    InvalidTargetKey { key: String },
    #[error("Invalid source key format: {key}")]
    InvalidSourceKey { key: String },
    #[error("Validation rule failed: {rule}")]
    ValidationRuleFailed { rule: String },
}

/// Validation rules for extracted values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    /// Value must not be empty
    NotEmpty,
    /// Value must be a valid integer
    IsInteger,
    /// Value must be a valid boolean
    IsBoolean,
    /// Value must match a regex pattern
    MatchesRegex(String),
    /// Value must be within a range (for numbers)
    InRange { min: Option<i64>, max: Option<i64> },
    /// Value must be one of the allowed values
    AllowedValues(Vec<String>),
    /// Value must have a minimum length
    MinLength(usize),
    /// Value must have a maximum length
    MaxLength(usize),
    /// Custom validation function name
    Custom(String),
}

impl ValidationRule {
    /// Apply the validation rule to a value
    pub fn validate(&self, value: &serde_json::Value) -> Result<(), ValidationError> {
        match self {
            ValidationRule::NotEmpty => match value {
                serde_json::Value::String(s) if s.is_empty() => {
                    Err(ValidationError::ValidationRuleFailed {
                        rule: "Value must not be empty".to_string(),
                    })
                }
                serde_json::Value::Null => Err(ValidationError::ValidationRuleFailed {
                    rule: "Value must not be null".to_string(),
                }),
                _ => Ok(()),
            },
            ValidationRule::IsInteger => match value {
                serde_json::Value::Number(_) => Ok(()),
                serde_json::Value::String(s) => s.parse::<i64>().map(|_| ()).map_err(|_| {
                    ValidationError::ValidationRuleFailed {
                        rule: "Value must be a valid integer".to_string(),
                    }
                }),
                _ => Err(ValidationError::ValidationRuleFailed {
                    rule: "Value must be a valid integer".to_string(),
                }),
            },
            ValidationRule::IsBoolean => match value {
                serde_json::Value::Bool(_) => Ok(()),
                serde_json::Value::String(s) => match s.to_lowercase().as_str() {
                    "true" | "false" => Ok(()),
                    _ => Err(ValidationError::ValidationRuleFailed {
                        rule: "Value must be a valid boolean".to_string(),
                    }),
                },
                _ => Err(ValidationError::ValidationRuleFailed {
                    rule: "Value must be a valid boolean".to_string(),
                }),
            },
            ValidationRule::MatchesRegex(pattern) => {
                match value {
                    serde_json::Value::String(s) => {
                        // Note: In a real implementation, you'd use a regex crate
                        // For now, we'll do a simple string check
                        if pattern == ".*" || s.contains(pattern) {
                            Ok(())
                        } else {
                            Err(ValidationError::ValidationRuleFailed {
                                rule: format!("Value must match pattern: {}", pattern),
                            })
                        }
                    }
                    _ => Err(ValidationError::ValidationRuleFailed {
                        rule: format!("Value must match pattern: {}", pattern),
                    }),
                }
            }
            ValidationRule::InRange { min, max } => match value {
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        let min_ok = min.is_none_or(|m| i >= m);
                        let max_ok = max.is_none_or(|m| i <= m);
                        if min_ok && max_ok {
                            Ok(())
                        } else {
                            Err(ValidationError::ValidationRuleFailed {
                                rule: format!("Value must be in range [{:?}, {:?}]", min, max),
                            })
                        }
                    } else {
                        Err(ValidationError::ValidationRuleFailed {
                            rule: "Value must be a number".to_string(),
                        })
                    }
                }
                _ => Err(ValidationError::ValidationRuleFailed {
                    rule: "Value must be a number".to_string(),
                }),
            },
            ValidationRule::AllowedValues(allowed) => match value {
                serde_json::Value::String(s) => {
                    if allowed.contains(s) {
                        Ok(())
                    } else {
                        Err(ValidationError::ValidationRuleFailed {
                            rule: format!("Value must be one of: {:?}", allowed),
                        })
                    }
                }
                _ => Err(ValidationError::ValidationRuleFailed {
                    rule: format!("Value must be one of: {:?}", allowed),
                }),
            },
            ValidationRule::MinLength(min_len) => match value {
                serde_json::Value::String(s) => {
                    if s.len() >= *min_len {
                        Ok(())
                    } else {
                        Err(ValidationError::ValidationRuleFailed {
                            rule: format!("Value must have minimum length: {}", min_len),
                        })
                    }
                }
                _ => Err(ValidationError::ValidationRuleFailed {
                    rule: format!("Value must have minimum length: {}", min_len),
                }),
            },
            ValidationRule::MaxLength(max_len) => match value {
                serde_json::Value::String(s) => {
                    if s.len() <= *max_len {
                        Ok(())
                    } else {
                        Err(ValidationError::ValidationRuleFailed {
                            rule: format!("Value must have maximum length: {}", max_len),
                        })
                    }
                }
                _ => Err(ValidationError::ValidationRuleFailed {
                    rule: format!("Value must have maximum length: {}", max_len),
                }),
            },
            ValidationRule::Custom(func_name) => Err(ValidationError::ValidationRuleFailed {
                rule: format!("Custom validation '{}' not implemented", func_name),
            }),
        }
    }
}

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

impl ValueMapping {
    /// Validate the value mapping configuration
    pub fn validate_config(&self) -> Result<(), ValidationError> {
        // Validate target key format
        if self.target_key.is_empty() {
            return Err(ValidationError::InvalidTargetKey {
                key: self.target_key.clone(),
            });
        }

        // Validate source key format
        if self.source_key.is_empty() {
            return Err(ValidationError::InvalidSourceKey {
                key: self.source_key.clone(),
            });
        }

        // Validate transformation if present
        if let Some(transform) = &self.transform
            && let ValueTransform::Custom(func_name) = transform
        {
            return Err(ValidationError::InvalidTransformation {
                transform: func_name.clone(),
            });
        }

        // Validate condition if present
        if let Some(condition) = &self.condition {
            match condition {
                Condition::Equals(key, _)
                | Condition::NotEquals(key, _)
                | Condition::Contains(key, _)
                | Condition::IsTruthy(key)
                | Condition::IsFalsy(key)
                | Condition::Exists(key)
                | Condition::NotExists(key) => {
                    if key.is_empty() {
                        return Err(ValidationError::InvalidCondition {
                            condition: format!("Empty key in condition: {:?}", condition),
                        });
                    }
                }
            }
        }

        // Validate validation rules if present
        for rule in &self.validation_rules {
            if let ValidationRule::Custom(func_name) = rule {
                return Err(ValidationError::ValidationRuleFailed {
                    rule: format!("Custom validation '{}' not implemented", func_name),
                });
            }
        }

        Ok(())
    }

    /// Validate an extracted value against the validation rules
    pub fn validate_value(&self, value: &serde_json::Value) -> Result<(), ValidationError> {
        for rule in &self.validation_rules {
            rule.validate(value)?;
        }
        Ok(())
    }

    /// Check for circular dependencies in conditions
    pub fn check_circular_dependencies(
        &self,
        all_mappings: &[ValueMapping],
    ) -> Result<(), ValidationError> {
        if let Some(condition) = &self.condition {
            let mut visited = std::collections::HashSet::new();
            let mut path = Vec::new();
            self.check_dependency_cycle(condition, all_mappings, &mut visited, &mut path)?;
        }
        Ok(())
    }

    fn check_dependency_cycle(
        &self,
        condition: &Condition,
        all_mappings: &[ValueMapping],
        visited: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
    ) -> Result<(), ValidationError> {
        let dependent_key = match condition {
            Condition::Equals(key, _)
            | Condition::NotEquals(key, _)
            | Condition::Contains(key, _)
            | Condition::IsTruthy(key)
            | Condition::IsFalsy(key)
            | Condition::Exists(key)
            | Condition::NotExists(key) => key,
        };

        if path.contains(dependent_key) {
            let mut cycle_path = path.clone();
            cycle_path.push(dependent_key.clone());
            return Err(ValidationError::CircularDependency {
                dependency_chain: cycle_path.join(" -> "),
            });
        }

        if visited.contains(dependent_key) {
            return Ok(());
        }

        visited.insert(dependent_key.clone());
        path.push(dependent_key.clone());

        // Find the mapping that produces this dependent key
        for mapping in all_mappings {
            if mapping.target_key == *dependent_key {
                if let Some(dep_condition) = &mapping.condition {
                    mapping.check_dependency_cycle(dep_condition, all_mappings, visited, path)?;
                }
                break;
            }
        }

        path.pop();
        Ok(())
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
    /// Validation rules to apply to the extracted value
    #[serde(default)]
    pub validation_rules: Vec<ValidationRule>,
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
            ValueTransform::ToBool => {
                let lower_value = value.to_lowercase();
                Ok(json!(lower_value == "true" || lower_value == "1"))
            }
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
        let mut validation_errors = Vec::new();

        // Validate all mappings before processing
        for mapping in &self.value_mappings {
            if let Err(e) = mapping.validate_config() {
                validation_errors.push(format!(
                    "Config validation failed for {}: {}",
                    mapping.target_key, e
                ));
            }
            if let Err(e) = mapping.check_circular_dependencies(&self.value_mappings) {
                validation_errors.push(format!(
                    "Circular dependency detected for {}: {}",
                    mapping.target_key, e
                ));
            }
        }

        // Log validation errors but continue processing
        for error in &validation_errors {
            eprintln!("Validation Error: {}", error);
        }

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
                                    // Validate the transformed value
                                    if let Err(e) = mapping.validate_value(&transformed) {
                                        eprintln!(
                                            "Warning: Value validation failed for {}: {}",
                                            mapping.target_key, e
                                        );
                                        // Continue processing even if validation fails
                                    }
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
                            let value_json = json!(value);
                            // Validate the raw value
                            if let Err(e) = mapping.validate_value(&value_json) {
                                eprintln!(
                                    "Warning: Value validation failed for {}: {}",
                                    mapping.target_key, e
                                );
                                // Continue processing even if validation fails
                            }
                            extracted.insert(mapping.target_key.clone(), value_json);
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
            facets: HashMap::new(),
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
        // Amp detection
        EnvMapping {
            id: "amp".to_string(),
            confidence: HIGH,
            indicators: vec![EnvIndicator {
                key: "AGENT".to_string(),
                value: Some("amp".to_string()),
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
        // Neovim detection (works for both :terminal and :!command modes)
        EnvMapping {
            id: "nvim".to_string(),
            confidence: HIGH,
            indicators: vec![
                EnvIndicator {
                    key: "NVIM".to_string(),
                    value: None,
                    required: false, // Optional - present in :terminal mode
                    prefix: false,
                    contains: None,
                    priority: 4,
                },
                EnvIndicator {
                    key: "VIMRUNTIME".to_string(),
                    value: None,
                    required: false, // Optional - present in :!command mode
                    prefix: false,
                    contains: None,
                    priority: 4,
                },
                EnvIndicator {
                    key: "MYVIMRC".to_string(),
                    value: None,
                    required: false, // Optional - present in both modes
                    prefix: false,
                    contains: None,
                    priority: 4,
                },
            ],
            facets: HashMap::from([("ide_id".to_string(), "nvim".to_string())]),
            contexts: vec!["ide".to_string()],
            value_mappings: vec![],
        },
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
    // Host mappings removed - host concept deprecated in favor of agent/ide detection
    vec![]
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
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "is_pr".to_string(),
                    source_key: "GITHUB_EVENT_NAME".to_string(),
                    required: false,
                    transform: Some(ValueTransform::Equals("pull_request".to_string())),
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "pr_number".to_string(),
                    source_key: "GITHUB_EVENT_NUMBER".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToInt),
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "repository".to_string(),
                    source_key: "GITHUB_REPOSITORY".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "workflow".to_string(),
                    source_key: "GITHUB_WORKFLOW".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
                },
                // Fallback branch detection for GitHub Actions
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "BRANCH_NAME".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "GIT_BRANCH".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
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
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "is_pr".to_string(),
                    source_key: "CI_MERGE_REQUEST_ID".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToBool),
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "pipeline_id".to_string(),
                    source_key: "CI_PIPELINE_ID".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToInt),
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "project_path".to_string(),
                    source_key: "CI_PROJECT_PATH".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
                },
                // Fallback branch detection for GitLab CI
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "BRANCH_NAME".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "GIT_BRANCH".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
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
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "is_pr".to_string(),
                    source_key: "CIRCLE_PR_NUMBER".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToBool),
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "build_number".to_string(),
                    source_key: "CIRCLE_BUILD_NUM".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToInt),
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "project_name".to_string(),
                    source_key: "CIRCLE_PROJECT_REPONAME".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
                },
                // Fallback branch detection for CircleCI
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "BRANCH_NAME".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "GIT_BRANCH".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
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
        // Generic CI detection for common environment variables
        EnvMapping {
            id: "generic-ci".to_string(),
            confidence: LOW,
            indicators: vec![EnvIndicator {
                key: "CI".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            }],
            facets: HashMap::from([("ci_id".to_string(), "generic".to_string())]),
            contexts: vec!["ci".to_string()],
            value_mappings: vec![
                ValueMapping {
                    target_key: "is_pr".to_string(),
                    source_key: "CI_PULL_REQUEST".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToBool),
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "BRANCH_NAME".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
                },
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "GIT_BRANCH".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
                },
            ],
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
    fn test_amp_mapping() {
        let mappings = get_agent_mappings();
        let amp_mapping = mappings.iter().find(|m| m.id == "amp").unwrap();

        let env_vars = HashMap::from([("AGENT".to_string(), "amp".to_string())]);

        assert!(amp_mapping.matches(&env_vars));
        assert_eq!(amp_mapping.confidence, HIGH);
    }

    #[test]
    fn test_value_transform_to_bool() {
        let transform = ValueTransform::ToBool;

        assert_eq!(transform.apply("").unwrap(), json!(false));
        assert_eq!(transform.apply("false").unwrap(), json!(false));
        assert_eq!(transform.apply("FALSE").unwrap(), json!(false));
        assert_eq!(transform.apply("value").unwrap(), json!(false));
        assert_eq!(transform.apply("123").unwrap(), json!(false));
        assert_eq!(transform.apply("true").unwrap(), json!(true));
        assert_eq!(transform.apply("TRUE").unwrap(), json!(true));
        assert_eq!(transform.apply("1").unwrap(), json!(true));
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
        assert_eq!(extracted.get("is_pr").unwrap(), &json!(false)); // Only "true" or "1" = true
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
        assert_eq!(extracted.get("is_pr").unwrap(), &json!(false)); // Only "true" or "1" = true
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
                    validation_rules: vec![],
                },
                // Then, extract pr_number only if is_pr is true
                ValueMapping {
                    target_key: "pr_number".to_string(),
                    source_key: "GITHUB_EVENT_NUMBER".to_string(),
                    required: false,
                    transform: Some(ValueTransform::ToInt),
                    condition: Some(Condition::IsTruthy("is_pr".to_string())),
                    validation_rules: vec![],
                },
                // Extract branch name regardless
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "GITHUB_REF_NAME".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
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

    #[test]
    fn test_validation_rule_not_empty() {
        let rule = ValidationRule::NotEmpty;

        // Valid cases
        assert!(rule.validate(&json!("hello")).is_ok());
        assert!(rule.validate(&json!(42)).is_ok());
        assert!(rule.validate(&json!(true)).is_ok());

        // Invalid cases
        assert!(rule.validate(&json!("")).is_err());
        assert!(rule.validate(&json!(serde_json::Value::Null)).is_err());
    }

    #[test]
    fn test_validation_rule_is_integer() {
        let rule = ValidationRule::IsInteger;

        // Valid cases
        assert!(rule.validate(&json!(42)).is_ok());
        assert!(rule.validate(&json!("123")).is_ok());
        assert!(rule.validate(&json!("-456")).is_ok());

        // Invalid cases
        assert!(rule.validate(&json!("not_a_number")).is_err());
        assert!(rule.validate(&json!("12.34")).is_err());
        assert!(rule.validate(&json!("hello")).is_err());
    }

    #[test]
    fn test_validation_rule_is_boolean() {
        let rule = ValidationRule::IsBoolean;

        // Valid cases
        assert!(rule.validate(&json!(true)).is_ok());
        assert!(rule.validate(&json!(false)).is_ok());
        assert!(rule.validate(&json!("true")).is_ok());
        assert!(rule.validate(&json!("false")).is_ok());

        // Invalid cases
        assert!(rule.validate(&json!("yes")).is_err());
        assert!(rule.validate(&json!("no")).is_err());
        assert!(rule.validate(&json!(42)).is_err());
    }

    #[test]
    fn test_validation_rule_in_range() {
        let rule = ValidationRule::InRange {
            min: Some(1),
            max: Some(100),
        };

        // Valid cases
        assert!(rule.validate(&json!(50)).is_ok());
        assert!(rule.validate(&json!(1)).is_ok());
        assert!(rule.validate(&json!(100)).is_ok());

        // Invalid cases
        assert!(rule.validate(&json!(0)).is_err());
        assert!(rule.validate(&json!(101)).is_err());
        assert!(rule.validate(&json!("50")).is_err());
    }

    #[test]
    fn test_validation_rule_allowed_values() {
        let rule = ValidationRule::AllowedValues(vec!["main".to_string(), "develop".to_string()]);

        // Valid cases
        assert!(rule.validate(&json!("main")).is_ok());
        assert!(rule.validate(&json!("develop")).is_ok());

        // Invalid cases
        assert!(rule.validate(&json!("feature")).is_err());
        assert!(rule.validate(&json!(42)).is_err());
    }

    #[test]
    fn test_validation_rule_length_constraints() {
        let min_rule = ValidationRule::MinLength(3);
        let max_rule = ValidationRule::MaxLength(10);

        // Valid cases
        assert!(min_rule.validate(&json!("hello")).is_ok());
        assert!(max_rule.validate(&json!("short")).is_ok());

        // Invalid cases
        assert!(min_rule.validate(&json!("hi")).is_err());
        assert!(max_rule.validate(&json!("very_long_string")).is_err());
    }

    #[test]
    fn test_value_mapping_validation() {
        let mapping = ValueMapping {
            target_key: "test".to_string(),
            source_key: "TEST_ENV".to_string(),
            required: false,
            transform: None,
            condition: None,
            validation_rules: vec![ValidationRule::NotEmpty, ValidationRule::MinLength(3)],
        };

        // Valid value
        assert!(mapping.validate_value(&json!("hello")).is_ok());

        // Invalid values
        assert!(mapping.validate_value(&json!("")).is_err()); // Empty
        assert!(mapping.validate_value(&json!("hi")).is_err()); // Too short
    }

    #[test]
    fn test_value_mapping_config_validation() {
        // Valid mapping
        let valid_mapping = ValueMapping {
            target_key: "test".to_string(),
            source_key: "TEST_ENV".to_string(),
            required: false,
            transform: None,
            condition: None,
            validation_rules: vec![],
        };
        assert!(valid_mapping.validate_config().is_ok());

        // Invalid mapping - empty target key
        let invalid_mapping = ValueMapping {
            target_key: "".to_string(),
            source_key: "TEST_ENV".to_string(),
            required: false,
            transform: None,
            condition: None,
            validation_rules: vec![],
        };
        assert!(invalid_mapping.validate_config().is_err());

        // Invalid mapping - empty source key
        let invalid_mapping2 = ValueMapping {
            target_key: "test".to_string(),
            source_key: "".to_string(),
            required: false,
            transform: None,
            condition: None,
            validation_rules: vec![],
        };
        assert!(invalid_mapping2.validate_config().is_err());
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mappings = vec![
            ValueMapping {
                target_key: "a".to_string(),
                source_key: "A_ENV".to_string(),
                required: false,
                transform: None,
                condition: Some(Condition::IsTruthy("b".to_string())),
                validation_rules: vec![],
            },
            ValueMapping {
                target_key: "b".to_string(),
                source_key: "B_ENV".to_string(),
                required: false,
                transform: None,
                condition: Some(Condition::IsTruthy("a".to_string())),
                validation_rules: vec![],
            },
        ];

        // Should detect circular dependency
        assert!(mappings[0].check_circular_dependencies(&mappings).is_err());
        assert!(mappings[1].check_circular_dependencies(&mappings).is_err());
    }

    #[test]
    fn test_validation_in_extract_values() {
        let mapping = EnvMapping {
            id: "test-validation".to_string(),
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
                ValueMapping {
                    target_key: "valid_value".to_string(),
                    source_key: "VALID_ENV".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![ValidationRule::NotEmpty],
                },
                ValueMapping {
                    target_key: "invalid_value".to_string(),
                    source_key: "INVALID_ENV".to_string(),
                    required: false,
                    transform: None,
                    condition: None,
                    validation_rules: vec![ValidationRule::MinLength(5)],
                },
            ],
        };

        let env_vars = HashMap::from([
            ("VALID_ENV".to_string(), "hello".to_string()),
            ("INVALID_ENV".to_string(), "hi".to_string()), // Too short
        ]);

        let extracted = mapping.extract_values(&env_vars);

        // Both values should be extracted (validation failures are logged but don't prevent extraction)
        assert_eq!(extracted.get("valid_value"), Some(&json!("hello")));
        assert_eq!(extracted.get("invalid_value"), Some(&json!("hi")));
    }
}
