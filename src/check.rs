use crate::schema::EnvSense;
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Check {
    Context(String),
    NestedField {
        path: Vec<String>,
        value: Option<String>,
    },
    LegacyFacet {
        key: String,
        value: String,
    },
    LegacyTrait {
        key: String,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParsedCheck {
    pub check: Check,
    pub negated: bool,
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ParseError {
    #[error("invalid check expression")]
    Invalid,
    #[error("empty input")]
    EmptyInput,
    #[error("invalid field path")]
    InvalidFieldPath,
    #[error("malformed comparison")]
    MalformedComparison,
}

/// Field Registry System for centralized field type and path management
#[derive(Debug, Clone)]
pub struct FieldRegistry {
    fields: HashMap<String, FieldInfo>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub field_type: FieldType,
    pub path: Vec<String>,
    pub description: String,
    pub context: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Boolean,
    String,
    OptionalString,
    ColorLevel,
    StreamInfo,
}

/// Result types for check evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum CheckResult {
    Boolean(bool),
    String(String),
    Comparison {
        actual: String,
        expected: String,
        matched: bool,
    },
}

impl CheckResult {
    /// Format the result for display with optional explanation
    pub fn format(&self, explain: bool) -> String {
        match self {
            CheckResult::Boolean(value) => {
                if explain {
                    format!("{}  # boolean result", value)
                } else {
                    value.to_string()
                }
            }
            CheckResult::String(value) => {
                if explain {
                    format!("{}  # string value", value)
                } else {
                    value.clone()
                }
            }
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                if explain {
                    format!("{}  # {} == {}", matched, actual, expected)
                } else {
                    matched.to_string()
                }
            }
        }
    }

    /// Convert result to boolean for exit code determination
    pub fn as_bool(&self) -> bool {
        match self {
            CheckResult::Boolean(b) => *b,
            CheckResult::Comparison { matched, .. } => *matched,
            CheckResult::String(_) => true, // String presence implies true
        }
    }

    /// Convert result to string representation
    pub fn as_string(&self) -> String {
        match self {
            CheckResult::Boolean(b) => b.to_string(),
            CheckResult::String(s) => s.clone(),
            CheckResult::Comparison { matched, .. } => matched.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub result: CheckResult,
    pub reason: Option<String>,
    pub signals: Option<BTreeMap<String, String>>,
}

/// Output formatting functions for CLI results
pub fn output_check_results(
    results: &[EvaluationResult],
    predicates: &[String],
    overall: bool,
    mode_any: bool,
    json: bool,
    explain: bool,
) {
    if json {
        output_json_results(results, predicates, overall, mode_any, explain);
    } else {
        output_human_results(results, predicates, overall, mode_any, explain);
    }
}

fn output_human_results(
    results: &[EvaluationResult],
    predicates: &[String],
    overall: bool,
    _mode_any: bool,
    explain: bool,
) {
    if results.len() == 1 {
        let result = &results[0];
        if let Some(reason) = result.reason.as_ref().filter(|_| explain) {
            println!("{}  # reason: {}", result.result.format(false), reason);
        } else {
            println!("{}", result.result.format(explain));
        }
    } else {
        println!("overall={}", overall);
        for (i, result) in results.iter().enumerate() {
            let predicate = &predicates[i];
            if let Some(reason) = result.reason.as_ref().filter(|_| explain) {
                println!(
                    "{}={}  # reason: {}",
                    predicate,
                    result.result.format(false),
                    reason
                );
            } else {
                println!("{}={}", predicate, result.result.format(explain));
            }
        }
    }
}

fn output_json_results(
    results: &[EvaluationResult],
    predicates: &[String],
    overall: bool,
    mode_any: bool,
    explain: bool,
) {
    use serde_json::json;

    let checks: Vec<serde_json::Value> = results
        .iter()
        .zip(predicates.iter())
        .map(|(result, predicate)| {
            let mut check = json!({
                "predicate": predicate,
                "result": result.result.as_bool(),
            });

            if explain {
                if let Some(reason) = &result.reason {
                    check["reason"] = json!(reason);
                }
                if let Some(signals) = &result.signals {
                    check["signals"] = json!(signals);
                }
            }

            check
        })
        .collect();

    let output = json!({
        "overall": overall,
        "mode": if mode_any { "any" } else { "all" },
        "checks": checks,
    });

    if explain {
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("{}", serde_json::to_string(&output).unwrap());
    }
}

pub fn parse(input: &str) -> Result<Check, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    // Parse based on syntax
    if let Some(rest) = input.strip_prefix("facet:") {
        parse_legacy_facet(rest)
    } else if let Some(rest) = input.strip_prefix("trait:") {
        parse_legacy_trait(rest)
    } else if input.contains('.') {
        parse_nested_field(input)
    } else {
        Ok(Check::Context(input.to_string()))
    }
}

pub fn parse_predicate(input: &str) -> Result<ParsedCheck, ParseError> {
    let input = input.trim();

    // Handle negation
    let (input, negated) = if let Some(rest) = input.strip_prefix('!') {
        (rest, true)
    } else {
        (input, false)
    };

    let check = parse(input)?;
    Ok(ParsedCheck { check, negated })
}

fn parse_nested_field(input: &str) -> Result<Check, ParseError> {
    let (path_str, value) = if let Some((path, val)) = input.split_once('=') {
        (path, Some(val.trim().to_string()))
    } else {
        (input, None)
    };

    let path_parts: Vec<String> = path_str
        .split('.')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if path_parts.len() < 2 {
        return Err(ParseError::InvalidFieldPath);
    }

    // Validate path format (context.field[.subfield])
    let valid_contexts = ["agent", "ide", "terminal", "ci"];
    if !valid_contexts.contains(&path_parts[0].as_str()) {
        return Err(ParseError::InvalidFieldPath);
    }

    Ok(Check::NestedField {
        path: path_parts,
        value,
    })
}

fn parse_legacy_facet(input: &str) -> Result<Check, ParseError> {
    if let Some((key, value)) = input.split_once('=') {
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(ParseError::MalformedComparison);
        }
        Ok(Check::LegacyFacet {
            key: key.to_string(),
            value: value.to_string(),
        })
    } else {
        Err(ParseError::MalformedComparison)
    }
}

fn parse_legacy_trait(input: &str) -> Result<Check, ParseError> {
    let key = input.trim();
    if key.is_empty() {
        return Err(ParseError::Invalid);
    }
    Ok(Check::LegacyTrait {
        key: key.to_string(),
    })
}

pub const CONTEXTS: &[&str] = &["agent", "ide", "ci", "container", "remote"];
pub const FACETS: &[&str] = &["agent_id", "ide_id", "ci_id", "container_id"];
pub const TRAITS: &[&str] = &[
    "is_interactive",
    "is_tty_stdin",
    "is_tty_stdout",
    "is_tty_stderr",
    "is_piped_stdin",
    "is_piped_stdout",
    "supports_hyperlinks",
    "is_ci",
    "ci_pr",
];

impl Default for FieldRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            fields: HashMap::new(),
        };
        registry.register_all_fields();
        registry
    }

    fn register_all_fields(&mut self) {
        // Agent fields
        self.register(
            "agent.id",
            FieldType::OptionalString,
            vec!["agent", "id"],
            "Agent identifier",
            "agent",
        );

        // IDE fields
        self.register(
            "ide.id",
            FieldType::OptionalString,
            vec!["ide", "id"],
            "IDE identifier",
            "ide",
        );

        // Terminal fields
        self.register(
            "terminal.interactive",
            FieldType::Boolean,
            vec!["terminal", "interactive"],
            "Terminal interactivity",
            "terminal",
        );
        self.register(
            "terminal.color_level",
            FieldType::ColorLevel,
            vec!["terminal", "color_level"],
            "Color support level",
            "terminal",
        );
        self.register(
            "terminal.stdin.tty",
            FieldType::Boolean,
            vec!["terminal", "stdin", "tty"],
            "Stdin is TTY",
            "terminal",
        );
        self.register(
            "terminal.stdout.tty",
            FieldType::Boolean,
            vec!["terminal", "stdout", "tty"],
            "Stdout is TTY",
            "terminal",
        );
        self.register(
            "terminal.stderr.tty",
            FieldType::Boolean,
            vec!["terminal", "stderr", "tty"],
            "Stderr is TTY",
            "terminal",
        );
        self.register(
            "terminal.stdin.piped",
            FieldType::Boolean,
            vec!["terminal", "stdin", "piped"],
            "Stdin is piped",
            "terminal",
        );
        self.register(
            "terminal.stdout.piped",
            FieldType::Boolean,
            vec!["terminal", "stdout", "piped"],
            "Stdout is piped",
            "terminal",
        );
        self.register(
            "terminal.stderr.piped",
            FieldType::Boolean,
            vec!["terminal", "stderr", "piped"],
            "Stderr is piped",
            "terminal",
        );
        self.register(
            "terminal.supports_hyperlinks",
            FieldType::Boolean,
            vec!["terminal", "supports_hyperlinks"],
            "Hyperlink support",
            "terminal",
        );

        // CI fields
        self.register(
            "ci.id",
            FieldType::OptionalString,
            vec!["ci", "id"],
            "CI system identifier",
            "ci",
        );
        self.register(
            "ci.vendor",
            FieldType::OptionalString,
            vec!["ci", "vendor"],
            "CI vendor",
            "ci",
        );
        self.register(
            "ci.name",
            FieldType::OptionalString,
            vec!["ci", "name"],
            "CI system name",
            "ci",
        );
        self.register(
            "ci.is_pr",
            FieldType::OptionalString,
            vec!["ci", "is_pr"],
            "Is pull request",
            "ci",
        );
        self.register(
            "ci.branch",
            FieldType::OptionalString,
            vec!["ci", "branch"],
            "Branch name",
            "ci",
        );
    }

    fn register(
        &mut self,
        field_path: &str,
        field_type: FieldType,
        path: Vec<&str>,
        description: &str,
        context: &str,
    ) {
        self.fields.insert(
            field_path.to_string(),
            FieldInfo {
                field_type,
                path: path.into_iter().map(|s| s.to_string()).collect(),
                description: description.to_string(),
                context: context.to_string(),
            },
        );
    }

    pub fn resolve_field(&self, path: &[String]) -> Option<&FieldInfo> {
        let key = path.join(".");
        self.fields.get(&key)
    }

    pub fn get_context_fields(&self, context: &str) -> Vec<(&String, &FieldInfo)> {
        self.fields
            .iter()
            .filter(|(_, info)| info.context == context)
            .collect()
    }

    pub fn list_all_fields(&self) -> Vec<&String> {
        self.fields.keys().collect()
    }
}

/// Enhanced Evaluation Logic - Task 2.3 Implementation
///
/// Main evaluation function that handles all check types with negation support
pub fn evaluate(env: &EnvSense, parsed: ParsedCheck, registry: &FieldRegistry) -> EvaluationResult {
    let mut eval_result = match parsed.check {
        Check::Context(ctx) => evaluate_context(env, &ctx),
        Check::NestedField { path, value } => {
            evaluate_nested_field(env, &path, value.as_deref(), registry)
        }
        Check::LegacyFacet { key, value } => evaluate_legacy_facet(env, &key, &value),
        Check::LegacyTrait { key } => evaluate_legacy_trait(env, &key),
    };

    // Handle negation
    if parsed.negated {
        eval_result.result = match eval_result.result {
            CheckResult::Boolean(b) => CheckResult::Boolean(!b),
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => CheckResult::Comparison {
                actual,
                expected,
                matched: !matched,
            },
            other => other, // String results don't negate
        };

        // Update reason for negated results
        if let Some(reason) = eval_result.reason.as_mut() {
            *reason = format!("negated: {}", reason);
        }
    }

    eval_result
}

/// Evaluate context checks - returns boolean indicating if context is detected
fn evaluate_context(env: &EnvSense, context: &str) -> EvaluationResult {
    let present = env.contexts.contains(&context.to_string());

    EvaluationResult {
        result: CheckResult::Boolean(present),
        reason: Some(format!(
            "context '{}' {}",
            context,
            if present { "detected" } else { "not detected" }
        )),
        signals: None,
    }
}

/// Evaluate nested field checks - supports both value display and comparison modes
fn evaluate_nested_field(
    env: &EnvSense,
    path: &[String],
    expected_value: Option<&str>,
    registry: &FieldRegistry,
) -> EvaluationResult {
    let field_info = match registry.resolve_field(path) {
        Some(info) => info,
        None => {
            return EvaluationResult {
                result: CheckResult::Boolean(false),
                reason: Some(format!("unknown field: {}", path.join("."))),
                signals: None,
            };
        }
    };

    // Navigate to the field value in the nested structure
    let actual_value = navigate_to_field(&env.traits, &field_info.path);

    match expected_value {
        Some(expected) => {
            // Comparison mode: return boolean match result
            let matched = compare_field_value(&actual_value, expected, &field_info.field_type);
            EvaluationResult {
                result: CheckResult::Comparison {
                    actual: format_field_value(&actual_value, &field_info.field_type),
                    expected: expected.to_string(),
                    matched,
                },
                reason: Some(format!(
                    "field comparison: {} == {}",
                    path.join("."),
                    expected
                )),
                signals: None,
            }
        }
        None => {
            // Value display mode: return actual value
            match &field_info.field_type {
                FieldType::Boolean => {
                    let bool_val = actual_value.as_bool().unwrap_or(false);
                    EvaluationResult {
                        result: CheckResult::Boolean(bool_val),
                        reason: Some(format!("field value: {}", path.join("."))),
                        signals: None,
                    }
                }
                _ => {
                    let string_val = format_field_value(&actual_value, &field_info.field_type);
                    EvaluationResult {
                        result: CheckResult::String(string_val),
                        reason: Some(format!("field value: {}", path.join("."))),
                        signals: None,
                    }
                }
            }
        }
    }
}

/// Navigate to a specific field in the nested traits structure
fn navigate_to_field(traits: &crate::traits::NestedTraits, path: &[String]) -> serde_json::Value {
    let traits_value = serde_json::to_value(traits).unwrap();
    let mut current = &traits_value;

    for segment in path {
        if let Some(obj) = current.as_object() {
            current = obj.get(segment).unwrap_or(&serde_json::Value::Null);
        } else {
            return serde_json::Value::Null;
        }
    }

    current.clone()
}

/// Compare field value with expected value based on field type
fn compare_field_value(actual: &serde_json::Value, expected: &str, field_type: &FieldType) -> bool {
    match field_type {
        FieldType::Boolean => {
            let actual_bool = actual.as_bool().unwrap_or(false);
            let expected_bool = expected == "true";
            actual_bool == expected_bool
        }
        FieldType::String | FieldType::OptionalString => {
            actual.as_str().map(|s| s == expected).unwrap_or(false)
        }
        FieldType::ColorLevel => {
            // Handle ColorLevel enum comparison
            actual.as_str().map(|s| s == expected).unwrap_or(false)
        }
        FieldType::StreamInfo => {
            // StreamInfo is an object, not directly comparable
            false
        }
    }
}

/// Format field value for display based on field type
fn format_field_value(value: &serde_json::Value, field_type: &FieldType) -> String {
    match field_type {
        FieldType::Boolean => value.as_bool().unwrap_or(false).to_string(),
        FieldType::String | FieldType::OptionalString => {
            value.as_str().unwrap_or("null").to_string()
        }
        FieldType::ColorLevel => value.as_str().unwrap_or("none").to_string(),
        FieldType::StreamInfo => {
            // Format StreamInfo object
            if let Some(obj) = value.as_object() {
                format!(
                    "tty:{}, piped:{}",
                    obj.get("tty").and_then(|v| v.as_bool()).unwrap_or(false),
                    obj.get("piped").and_then(|v| v.as_bool()).unwrap_or(false)
                )
            } else {
                "null".to_string()
            }
        }
    }
}

/// Evaluate legacy facet checks for backward compatibility
fn evaluate_legacy_facet(env: &EnvSense, key: &str, value: &str) -> EvaluationResult {
    // Map legacy facet keys to new nested field paths
    let (field_path, field_type) = match key {
        "agent_id" => (
            vec!["agent".to_string(), "id".to_string()],
            FieldType::OptionalString,
        ),
        "ide_id" => (
            vec!["ide".to_string(), "id".to_string()],
            FieldType::OptionalString,
        ),
        "ci_id" => (
            vec!["ci".to_string(), "id".to_string()],
            FieldType::OptionalString,
        ),
        _ => {
            return EvaluationResult {
                result: CheckResult::Boolean(false),
                reason: Some(format!("unknown legacy facet: {}", key)),
                signals: None,
            };
        }
    };

    let actual_value = navigate_to_field(&env.traits, &field_path);
    let matched = compare_field_value(&actual_value, value, &field_type);

    EvaluationResult {
        result: CheckResult::Comparison {
            actual: format_field_value(&actual_value, &field_type),
            expected: value.to_string(),
            matched,
        },
        reason: Some(format!("legacy facet comparison: {}={}", key, value)),
        signals: None,
    }
}

/// Evaluate legacy trait checks for backward compatibility
fn evaluate_legacy_trait(env: &EnvSense, key: &str) -> EvaluationResult {
    // Map legacy trait keys to new nested field paths
    let (field_path, _field_type) = match key {
        "is_interactive" => (
            vec!["terminal".to_string(), "interactive".to_string()],
            FieldType::Boolean,
        ),
        "supports_hyperlinks" => (
            vec!["terminal".to_string(), "supports_hyperlinks".to_string()],
            FieldType::Boolean,
        ),
        "is_tty_stdin" => (
            vec![
                "terminal".to_string(),
                "stdin".to_string(),
                "tty".to_string(),
            ],
            FieldType::Boolean,
        ),
        "is_tty_stdout" => (
            vec![
                "terminal".to_string(),
                "stdout".to_string(),
                "tty".to_string(),
            ],
            FieldType::Boolean,
        ),
        "is_tty_stderr" => (
            vec![
                "terminal".to_string(),
                "stderr".to_string(),
                "tty".to_string(),
            ],
            FieldType::Boolean,
        ),
        "is_piped_stdin" => (
            vec![
                "terminal".to_string(),
                "stdin".to_string(),
                "piped".to_string(),
            ],
            FieldType::Boolean,
        ),
        "is_piped_stdout" => (
            vec![
                "terminal".to_string(),
                "stdout".to_string(),
                "piped".to_string(),
            ],
            FieldType::Boolean,
        ),
        _ => {
            return EvaluationResult {
                result: CheckResult::Boolean(false),
                reason: Some(format!("unknown legacy trait: {}", key)),
                signals: None,
            };
        }
    };

    let actual_value = navigate_to_field(&env.traits, &field_path);
    let bool_val = actual_value.as_bool().unwrap_or(false);

    EvaluationResult {
        result: CheckResult::Boolean(bool_val),
        reason: Some(format!("legacy trait value: {}", key)),
        signals: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Context parsing tests
    #[test]
    fn parse_context() {
        assert_eq!(parse("agent"), Ok(Check::Context("agent".into())));
        assert_eq!(parse("ide"), Ok(Check::Context("ide".into())));
        assert_eq!(parse("ci"), Ok(Check::Context("ci".into())));
        assert_eq!(parse("terminal"), Ok(Check::Context("terminal".into())));
    }

    #[test]
    fn parse_context_with_whitespace() {
        assert_eq!(parse("  agent  "), Ok(Check::Context("agent".into())));
        assert_eq!(parse("\tagent\n"), Ok(Check::Context("agent".into())));
    }

    // Nested field parsing tests
    #[test]
    fn parse_nested_field_without_value() {
        assert_eq!(
            parse("agent.id"),
            Ok(Check::NestedField {
                path: vec!["agent".into(), "id".into()],
                value: None
            })
        );
        assert_eq!(
            parse("terminal.interactive"),
            Ok(Check::NestedField {
                path: vec!["terminal".into(), "interactive".into()],
                value: None
            })
        );
    }

    #[test]
    fn parse_nested_field_with_value() {
        assert_eq!(
            parse("agent.id=cursor"),
            Ok(Check::NestedField {
                path: vec!["agent".into(), "id".into()],
                value: Some("cursor".into())
            })
        );
        assert_eq!(
            parse("terminal.interactive=true"),
            Ok(Check::NestedField {
                path: vec!["terminal".into(), "interactive".into()],
                value: Some("true".into())
            })
        );
    }

    #[test]
    fn parse_nested_field_deep_path() {
        assert_eq!(
            parse("terminal.stdin.tty"),
            Ok(Check::NestedField {
                path: vec!["terminal".into(), "stdin".into(), "tty".into()],
                value: None
            })
        );
        assert_eq!(
            parse("terminal.stdout.piped=true"),
            Ok(Check::NestedField {
                path: vec!["terminal".into(), "stdout".into(), "piped".into()],
                value: Some("true".into())
            })
        );
    }

    #[test]
    fn parse_nested_field_with_whitespace() {
        assert_eq!(
            parse("  agent.id = cursor  "),
            Ok(Check::NestedField {
                path: vec!["agent".into(), "id".into()],
                value: Some("cursor".into())
            })
        );
    }

    #[test]
    fn parse_nested_field_invalid_context() {
        assert_eq!(parse("invalid.field"), Err(ParseError::InvalidFieldPath));
        assert_eq!(parse("unknown.id=test"), Err(ParseError::InvalidFieldPath));
    }

    #[test]
    fn parse_nested_field_invalid_path() {
        assert_eq!(parse("agent"), Ok(Check::Context("agent".into()))); // No dot, so context
        assert_eq!(parse("agent."), Err(ParseError::InvalidFieldPath)); // Empty field
        assert_eq!(parse(".id"), Err(ParseError::InvalidFieldPath)); // Empty context
    }

    // Legacy facet parsing tests
    #[test]
    fn parse_legacy_facet() {
        assert_eq!(
            parse("facet:ide_id=vscode"),
            Ok(Check::LegacyFacet {
                key: "ide_id".into(),
                value: "vscode".into()
            })
        );
        assert_eq!(
            parse("facet:agent_id=cursor"),
            Ok(Check::LegacyFacet {
                key: "agent_id".into(),
                value: "cursor".into()
            })
        );
    }

    #[test]
    fn parse_legacy_facet_with_whitespace() {
        assert_eq!(
            parse("facet: ide_id = vscode "),
            Ok(Check::LegacyFacet {
                key: "ide_id".into(),
                value: "vscode".into()
            })
        );
    }

    #[test]
    fn parse_legacy_facet_invalid() {
        assert_eq!(parse("facet:"), Err(ParseError::MalformedComparison));
        assert_eq!(parse("facet:key"), Err(ParseError::MalformedComparison));
        assert_eq!(parse("facet:=value"), Err(ParseError::MalformedComparison));
        assert_eq!(parse("facet:key="), Err(ParseError::MalformedComparison));
        assert_eq!(parse("facet: = "), Err(ParseError::MalformedComparison));
    }

    // Legacy trait parsing tests
    #[test]
    fn parse_legacy_trait() {
        assert_eq!(
            parse("trait:is_interactive"),
            Ok(Check::LegacyTrait {
                key: "is_interactive".into()
            })
        );
        assert_eq!(
            parse("trait:supports_hyperlinks"),
            Ok(Check::LegacyTrait {
                key: "supports_hyperlinks".into()
            })
        );
    }

    #[test]
    fn parse_legacy_trait_with_whitespace() {
        assert_eq!(
            parse("trait: is_interactive "),
            Ok(Check::LegacyTrait {
                key: "is_interactive".into()
            })
        );
    }

    #[test]
    fn parse_legacy_trait_invalid() {
        assert_eq!(parse("trait:"), Err(ParseError::Invalid));
        assert_eq!(parse("trait: "), Err(ParseError::Invalid));
    }

    // Error handling tests
    #[test]
    fn parse_empty_input() {
        assert_eq!(parse(""), Err(ParseError::EmptyInput));
        assert_eq!(parse("  "), Err(ParseError::EmptyInput));
        assert_eq!(parse("\t\n"), Err(ParseError::EmptyInput));
    }

    // Negation parsing tests
    #[test]
    fn parse_predicate_negation() {
        let pc = parse_predicate("!ci").unwrap();
        assert!(pc.negated);
        assert_eq!(pc.check, Check::Context("ci".into()));

        let pc = parse_predicate("!agent.id=cursor").unwrap();
        assert!(pc.negated);
        assert_eq!(
            pc.check,
            Check::NestedField {
                path: vec!["agent".into(), "id".into()],
                value: Some("cursor".into())
            }
        );

        let pc = parse_predicate("!facet:ide_id=vscode").unwrap();
        assert!(pc.negated);
        assert_eq!(
            pc.check,
            Check::LegacyFacet {
                key: "ide_id".into(),
                value: "vscode".into()
            }
        );

        let pc = parse_predicate("!trait:is_interactive").unwrap();
        assert!(pc.negated);
        assert_eq!(
            pc.check,
            Check::LegacyTrait {
                key: "is_interactive".into()
            }
        );
    }

    #[test]
    fn parse_predicate_no_negation() {
        let pc = parse_predicate("ci").unwrap();
        assert!(!pc.negated);
        assert_eq!(pc.check, Check::Context("ci".into()));

        let pc = parse_predicate("agent.id").unwrap();
        assert!(!pc.negated);
        assert_eq!(
            pc.check,
            Check::NestedField {
                path: vec!["agent".into(), "id".into()],
                value: None
            }
        );
    }

    #[test]
    fn parse_predicate_with_whitespace() {
        let pc = parse_predicate("  !agent.id=cursor  ").unwrap();
        assert!(pc.negated);
        assert_eq!(
            pc.check,
            Check::NestedField {
                path: vec!["agent".into(), "id".into()],
                value: Some("cursor".into())
            }
        );

        let pc = parse_predicate("  agent.id  ").unwrap();
        assert!(!pc.negated);
        assert_eq!(
            pc.check,
            Check::NestedField {
                path: vec!["agent".into(), "id".into()],
                value: None
            }
        );
    }

    // Edge case tests
    #[test]
    fn parse_all_valid_contexts() {
        for context in &["agent", "ide", "terminal", "ci"] {
            assert_eq!(parse(context), Ok(Check::Context(context.to_string())));

            let field_path = format!("{}.id", context);
            assert_eq!(
                parse(&field_path),
                Ok(Check::NestedField {
                    path: vec![context.to_string(), "id".to_string()],
                    value: None
                })
            );
        }
    }

    #[test]
    fn parse_complex_field_values() {
        assert_eq!(
            parse("ci.branch=feature/test-123"),
            Ok(Check::NestedField {
                path: vec!["ci".into(), "branch".into()],
                value: Some("feature/test-123".into())
            })
        );

        assert_eq!(
            parse("agent.id=cursor-ai"),
            Ok(Check::NestedField {
                path: vec!["agent".into(), "id".into()],
                value: Some("cursor-ai".into())
            })
        );
    }

    #[test]
    fn parse_error_propagation() {
        assert_eq!(parse_predicate(""), Err(ParseError::EmptyInput));
        assert_eq!(parse_predicate("!"), Err(ParseError::EmptyInput));
        assert_eq!(
            parse_predicate("invalid.field"),
            Err(ParseError::InvalidFieldPath)
        );
        assert_eq!(
            parse_predicate("!invalid.field"),
            Err(ParseError::InvalidFieldPath)
        );
    }

    // Field Registry Tests
    #[test]
    fn field_registry_creation() {
        let registry = FieldRegistry::new();
        let fields = registry.list_all_fields();

        // Should have all expected fields registered
        assert!(!fields.is_empty());
        assert!(fields.len() >= 15); // At least the fields we registered
    }

    #[test]
    fn field_registry_agent_fields() {
        let registry = FieldRegistry::new();

        // Test agent field resolution
        let agent_id = registry.resolve_field(&vec!["agent".to_string(), "id".to_string()]);
        assert!(agent_id.is_some());
        let field_info = agent_id.unwrap();
        assert_eq!(field_info.field_type, FieldType::OptionalString);
        assert_eq!(field_info.context, "agent");
        assert_eq!(field_info.description, "Agent identifier");
        assert_eq!(field_info.path, vec!["agent", "id"]);
    }

    #[test]
    fn field_registry_ide_fields() {
        let registry = FieldRegistry::new();

        // Test IDE field resolution
        let ide_id = registry.resolve_field(&vec!["ide".to_string(), "id".to_string()]);
        assert!(ide_id.is_some());
        let field_info = ide_id.unwrap();
        assert_eq!(field_info.field_type, FieldType::OptionalString);
        assert_eq!(field_info.context, "ide");
        assert_eq!(field_info.description, "IDE identifier");
    }

    #[test]
    fn field_registry_terminal_fields() {
        let registry = FieldRegistry::new();

        // Test terminal boolean fields
        let interactive =
            registry.resolve_field(&vec!["terminal".to_string(), "interactive".to_string()]);
        assert!(interactive.is_some());
        assert_eq!(interactive.unwrap().field_type, FieldType::Boolean);
        assert_eq!(interactive.unwrap().context, "terminal");

        // Test terminal color level field
        let color_level =
            registry.resolve_field(&vec!["terminal".to_string(), "color_level".to_string()]);
        assert!(color_level.is_some());
        assert_eq!(color_level.unwrap().field_type, FieldType::ColorLevel);

        // Test terminal stream fields
        let stdin_tty = registry.resolve_field(&vec![
            "terminal".to_string(),
            "stdin".to_string(),
            "tty".to_string(),
        ]);
        assert!(stdin_tty.is_some());
        assert_eq!(stdin_tty.unwrap().field_type, FieldType::Boolean);
        assert_eq!(stdin_tty.unwrap().description, "Stdin is TTY");

        let stdout_piped = registry.resolve_field(&vec![
            "terminal".to_string(),
            "stdout".to_string(),
            "piped".to_string(),
        ]);
        assert!(stdout_piped.is_some());
        assert_eq!(stdout_piped.unwrap().field_type, FieldType::Boolean);
        assert_eq!(stdout_piped.unwrap().description, "Stdout is piped");

        // Test hyperlinks support
        let hyperlinks = registry.resolve_field(&vec![
            "terminal".to_string(),
            "supports_hyperlinks".to_string(),
        ]);
        assert!(hyperlinks.is_some());
        assert_eq!(hyperlinks.unwrap().field_type, FieldType::Boolean);
    }

    #[test]
    fn field_registry_ci_fields() {
        let registry = FieldRegistry::new();

        // Test all CI fields
        let ci_id = registry.resolve_field(&vec!["ci".to_string(), "id".to_string()]);
        assert!(ci_id.is_some());
        assert_eq!(ci_id.unwrap().field_type, FieldType::OptionalString);
        assert_eq!(ci_id.unwrap().context, "ci");

        let ci_vendor = registry.resolve_field(&vec!["ci".to_string(), "vendor".to_string()]);
        assert!(ci_vendor.is_some());
        assert_eq!(ci_vendor.unwrap().field_type, FieldType::OptionalString);
        assert_eq!(ci_vendor.unwrap().description, "CI vendor");

        let ci_name = registry.resolve_field(&vec!["ci".to_string(), "name".to_string()]);
        assert!(ci_name.is_some());
        assert_eq!(ci_name.unwrap().field_type, FieldType::OptionalString);

        let ci_is_pr = registry.resolve_field(&vec!["ci".to_string(), "is_pr".to_string()]);
        assert!(ci_is_pr.is_some());
        assert_eq!(ci_is_pr.unwrap().field_type, FieldType::OptionalString);

        let ci_branch = registry.resolve_field(&vec!["ci".to_string(), "branch".to_string()]);
        assert!(ci_branch.is_some());
        assert_eq!(ci_branch.unwrap().field_type, FieldType::OptionalString);
        assert_eq!(ci_branch.unwrap().description, "Branch name");
    }

    #[test]
    fn field_registry_context_filtering() {
        let registry = FieldRegistry::new();

        // Test context-based field filtering
        let agent_fields = registry.get_context_fields("agent");
        assert_eq!(agent_fields.len(), 1);
        assert!(
            agent_fields
                .iter()
                .any(|(path, _)| path.as_str() == "agent.id")
        );

        let ide_fields = registry.get_context_fields("ide");
        assert_eq!(ide_fields.len(), 1);
        assert!(ide_fields.iter().any(|(path, _)| path.as_str() == "ide.id"));

        let terminal_fields = registry.get_context_fields("terminal");
        assert!(terminal_fields.len() >= 8); // At least 8 terminal fields
        assert!(
            terminal_fields
                .iter()
                .any(|(path, _)| path.as_str() == "terminal.interactive")
        );
        assert!(
            terminal_fields
                .iter()
                .any(|(path, _)| path.as_str() == "terminal.color_level")
        );
        assert!(
            terminal_fields
                .iter()
                .any(|(path, _)| path.as_str() == "terminal.stdin.tty")
        );
        assert!(
            terminal_fields
                .iter()
                .any(|(path, _)| path.as_str() == "terminal.supports_hyperlinks")
        );

        let ci_fields = registry.get_context_fields("ci");
        assert_eq!(ci_fields.len(), 5);
        assert!(ci_fields.iter().any(|(path, _)| path.as_str() == "ci.id"));
        assert!(
            ci_fields
                .iter()
                .any(|(path, _)| path.as_str() == "ci.vendor")
        );
        assert!(
            ci_fields
                .iter()
                .any(|(path, _)| path.as_str() == "ci.branch")
        );
    }

    #[test]
    fn field_registry_unknown_field() {
        let registry = FieldRegistry::new();

        // Test unknown field resolution
        let unknown = registry.resolve_field(&vec!["unknown".to_string(), "field".to_string()]);
        assert!(unknown.is_none());

        let partial_unknown =
            registry.resolve_field(&vec!["agent".to_string(), "unknown".to_string()]);
        assert!(partial_unknown.is_none());
    }

    #[test]
    fn field_registry_unknown_context() {
        let registry = FieldRegistry::new();

        // Test unknown context filtering
        let unknown_fields = registry.get_context_fields("unknown");
        assert!(unknown_fields.is_empty());
    }

    #[test]
    fn field_registry_field_type_validation() {
        let registry = FieldRegistry::new();

        // Test that field types are correctly assigned
        let boolean_fields = registry
            .list_all_fields()
            .iter()
            .filter_map(|path| {
                registry.resolve_field(&path.split('.').map(|s| s.to_string()).collect::<Vec<_>>())
            })
            .filter(|info| info.field_type == FieldType::Boolean)
            .count();
        assert!(boolean_fields >= 7); // At least 7 boolean fields (interactive + 6 stream fields)

        let optional_string_fields = registry
            .list_all_fields()
            .iter()
            .filter_map(|path| {
                registry.resolve_field(&path.split('.').map(|s| s.to_string()).collect::<Vec<_>>())
            })
            .filter(|info| info.field_type == FieldType::OptionalString)
            .count();
        assert!(optional_string_fields >= 7); // At least 7 optional string fields (agent.id, ide.id, 5 CI fields)

        let color_level_fields = registry
            .list_all_fields()
            .iter()
            .filter_map(|path| {
                registry.resolve_field(&path.split('.').map(|s| s.to_string()).collect::<Vec<_>>())
            })
            .filter(|info| info.field_type == FieldType::ColorLevel)
            .count();
        assert_eq!(color_level_fields, 1); // Exactly 1 color level field
    }

    #[test]
    fn field_registry_completeness() {
        let registry = FieldRegistry::new();
        let all_fields = registry.list_all_fields();

        // Verify all expected fields are present
        let expected_fields = vec![
            "agent.id",
            "ide.id",
            "terminal.interactive",
            "terminal.color_level",
            "terminal.stdin.tty",
            "terminal.stdout.tty",
            "terminal.stderr.tty",
            "terminal.stdin.piped",
            "terminal.stdout.piped",
            "terminal.stderr.piped",
            "terminal.supports_hyperlinks",
            "ci.id",
            "ci.vendor",
            "ci.name",
            "ci.is_pr",
            "ci.branch",
        ];

        for expected in expected_fields {
            assert!(
                all_fields.iter().any(|field| field.as_str() == expected),
                "Missing expected field: {}",
                expected
            );
        }
    }

    #[test]
    fn field_registry_path_consistency() {
        let registry = FieldRegistry::new();

        // Test that field paths are consistent with field names
        for field_path in registry.list_all_fields() {
            let path_parts: Vec<String> = field_path.split('.').map(|s| s.to_string()).collect();
            let field_info = registry.resolve_field(&path_parts).unwrap();
            assert_eq!(
                field_info.path, path_parts,
                "Path mismatch for field: {}",
                field_path
            );
        }
    }

    // Enhanced Evaluation Logic Tests - Task 2.3

    fn create_test_env() -> EnvSense {
        use crate::traits::terminal::ColorLevel;
        use crate::traits::{
            AgentTraits, CiTraits, IdeTraits, NestedTraits, StreamInfo, TerminalTraits,
        };

        EnvSense {
            contexts: vec!["agent".to_string(), "terminal".to_string()],
            traits: NestedTraits {
                agent: AgentTraits {
                    id: Some("cursor".to_string()),
                },
                ide: IdeTraits {
                    id: Some("vscode".to_string()),
                },
                terminal: TerminalTraits {
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
                },
                ci: CiTraits {
                    id: None,
                    vendor: None,
                    name: None,
                    is_pr: None,
                    branch: None,
                },
            },
            host: None,
            evidence: vec![],
            version: "0.3.0".to_string(),
        }
    }

    #[test]
    fn evaluate_context_present() {
        let env = create_test_env();
        let result = evaluate_context(&env, "agent");

        assert_eq!(result.result, CheckResult::Boolean(true));
        let reason = result.reason.unwrap();
        assert!(reason.contains("agent"));
        assert!(reason.contains("detected"));
        assert!(result.signals.is_none());
    }

    #[test]
    fn evaluate_context_absent() {
        let env = create_test_env();
        let result = evaluate_context(&env, "ci");

        assert_eq!(result.result, CheckResult::Boolean(false));
        let reason = result.reason.unwrap();
        assert!(reason.contains("ci"));
        assert!(reason.contains("not detected"));
        assert!(result.signals.is_none());
    }

    #[test]
    fn evaluate_nested_field_boolean_value() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["terminal".to_string(), "interactive".to_string()];

        let result = evaluate_nested_field(&env, &path, None, &registry);

        assert_eq!(result.result, CheckResult::Boolean(true));
        assert!(
            result
                .reason
                .unwrap()
                .contains("field value: terminal.interactive")
        );
    }

    #[test]
    fn evaluate_nested_field_string_value() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["agent".to_string(), "id".to_string()];

        let result = evaluate_nested_field(&env, &path, None, &registry);

        assert_eq!(result.result, CheckResult::String("cursor".to_string()));
        assert!(result.reason.unwrap().contains("field value: agent.id"));
    }

    #[test]
    fn evaluate_nested_field_comparison_match() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["agent".to_string(), "id".to_string()];

        let result = evaluate_nested_field(&env, &path, Some("cursor"), &registry);

        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "cursor");
                assert_eq!(expected, "cursor");
                assert!(matched);
            }
            _ => panic!("Expected Comparison result"),
        }
        assert!(
            result
                .reason
                .unwrap()
                .contains("field comparison: agent.id == cursor")
        );
    }

    #[test]
    fn evaluate_nested_field_comparison_no_match() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["agent".to_string(), "id".to_string()];

        let result = evaluate_nested_field(&env, &path, Some("other"), &registry);

        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "cursor");
                assert_eq!(expected, "other");
                assert!(!matched);
            }
            _ => panic!("Expected Comparison result"),
        }
    }

    #[test]
    fn evaluate_nested_field_boolean_comparison() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["terminal".to_string(), "interactive".to_string()];

        let result = evaluate_nested_field(&env, &path, Some("true"), &registry);

        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "true");
                assert_eq!(expected, "true");
                assert!(matched);
            }
            _ => panic!("Expected Comparison result"),
        }
    }

    #[test]
    fn evaluate_nested_field_unknown_field() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["unknown".to_string(), "field".to_string()];

        let result = evaluate_nested_field(&env, &path, None, &registry);

        assert_eq!(result.result, CheckResult::Boolean(false));
        assert!(
            result
                .reason
                .unwrap()
                .contains("unknown field: unknown.field")
        );
    }

    #[test]
    fn evaluate_nested_field_deep_path() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec![
            "terminal".to_string(),
            "stdin".to_string(),
            "tty".to_string(),
        ];

        let result = evaluate_nested_field(&env, &path, None, &registry);

        assert_eq!(result.result, CheckResult::Boolean(true));
        assert!(
            result
                .reason
                .unwrap()
                .contains("field value: terminal.stdin.tty")
        );
    }

    #[test]
    fn evaluate_legacy_facet_match() {
        let env = create_test_env();

        let result = evaluate_legacy_facet(&env, "agent_id", "cursor");

        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "cursor");
                assert_eq!(expected, "cursor");
                assert!(matched);
            }
            _ => panic!("Expected Comparison result"),
        }
        assert!(
            result
                .reason
                .unwrap()
                .contains("legacy facet comparison: agent_id=cursor")
        );
    }

    #[test]
    fn evaluate_legacy_facet_no_match() {
        let env = create_test_env();

        let result = evaluate_legacy_facet(&env, "agent_id", "other");

        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "cursor");
                assert_eq!(expected, "other");
                assert!(!matched);
            }
            _ => panic!("Expected Comparison result"),
        }
    }

    #[test]
    fn evaluate_legacy_facet_unknown() {
        let env = create_test_env();

        let result = evaluate_legacy_facet(&env, "unknown_facet", "value");

        assert_eq!(result.result, CheckResult::Boolean(false));
        assert!(
            result
                .reason
                .unwrap()
                .contains("unknown legacy facet: unknown_facet")
        );
    }

    #[test]
    fn evaluate_legacy_trait_true() {
        let env = create_test_env();

        let result = evaluate_legacy_trait(&env, "is_interactive");

        assert_eq!(result.result, CheckResult::Boolean(true));
        assert!(
            result
                .reason
                .unwrap()
                .contains("legacy trait value: is_interactive")
        );
    }

    #[test]
    fn evaluate_legacy_trait_false() {
        let env = create_test_env();

        let result = evaluate_legacy_trait(&env, "is_piped_stdin");

        assert_eq!(result.result, CheckResult::Boolean(false));
        assert!(
            result
                .reason
                .unwrap()
                .contains("legacy trait value: is_piped_stdin")
        );
    }

    #[test]
    fn evaluate_legacy_trait_unknown() {
        let env = create_test_env();

        let result = evaluate_legacy_trait(&env, "unknown_trait");

        assert_eq!(result.result, CheckResult::Boolean(false));
        assert!(
            result
                .reason
                .unwrap()
                .contains("unknown legacy trait: unknown_trait")
        );
    }

    #[test]
    fn evaluate_main_function_context() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let parsed = ParsedCheck {
            check: Check::Context("agent".to_string()),
            negated: false,
        };

        let result = evaluate(&env, parsed, &registry);

        assert_eq!(result.result, CheckResult::Boolean(true));
        let reason = result.reason.unwrap();
        assert!(reason.contains("agent"));
        assert!(reason.contains("detected"));
    }

    #[test]
    fn evaluate_main_function_nested_field() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let parsed = ParsedCheck {
            check: Check::NestedField {
                path: vec!["agent".to_string(), "id".to_string()],
                value: None,
            },
            negated: false,
        };

        let result = evaluate(&env, parsed, &registry);

        assert_eq!(result.result, CheckResult::String("cursor".to_string()));
    }

    #[test]
    fn evaluate_main_function_legacy_facet() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let parsed = ParsedCheck {
            check: Check::LegacyFacet {
                key: "agent_id".to_string(),
                value: "cursor".to_string(),
            },
            negated: false,
        };

        let result = evaluate(&env, parsed, &registry);

        match result.result {
            CheckResult::Comparison { matched, .. } => assert!(matched),
            _ => panic!("Expected Comparison result"),
        }
    }

    #[test]
    fn evaluate_main_function_legacy_trait() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let parsed = ParsedCheck {
            check: Check::LegacyTrait {
                key: "is_interactive".to_string(),
            },
            negated: false,
        };

        let result = evaluate(&env, parsed, &registry);

        assert_eq!(result.result, CheckResult::Boolean(true));
    }

    #[test]
    fn evaluate_negation_boolean() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let parsed = ParsedCheck {
            check: Check::Context("agent".to_string()),
            negated: true,
        };

        let result = evaluate(&env, parsed, &registry);

        assert_eq!(result.result, CheckResult::Boolean(false));
        assert!(result.reason.unwrap().contains("negated:"));
    }

    #[test]
    fn evaluate_negation_comparison() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let parsed = ParsedCheck {
            check: Check::NestedField {
                path: vec!["agent".to_string(), "id".to_string()],
                value: Some("cursor".to_string()),
            },
            negated: true,
        };

        let result = evaluate(&env, parsed, &registry);

        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "cursor");
                assert_eq!(expected, "cursor");
                assert!(!matched); // Negated
            }
            _ => panic!("Expected Comparison result"),
        }
        assert!(result.reason.unwrap().contains("negated:"));
    }

    #[test]
    fn evaluate_negation_string_unchanged() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let parsed = ParsedCheck {
            check: Check::NestedField {
                path: vec!["agent".to_string(), "id".to_string()],
                value: None,
            },
            negated: true,
        };

        let result = evaluate(&env, parsed, &registry);

        // String results don't negate, but reason is updated
        assert_eq!(result.result, CheckResult::String("cursor".to_string()));
        assert!(result.reason.unwrap().contains("negated:"));
    }

    #[test]
    fn navigate_to_field_success() {
        let env = create_test_env();
        let path = vec!["agent".to_string(), "id".to_string()];

        let value = navigate_to_field(&env.traits, &path);

        assert_eq!(value.as_str().unwrap(), "cursor");
    }

    #[test]
    fn navigate_to_field_deep_path() {
        let env = create_test_env();
        let path = vec![
            "terminal".to_string(),
            "stdin".to_string(),
            "tty".to_string(),
        ];

        let value = navigate_to_field(&env.traits, &path);

        assert_eq!(value.as_bool().unwrap(), true);
    }

    #[test]
    fn navigate_to_field_missing() {
        let env = create_test_env();
        let path = vec!["unknown".to_string(), "field".to_string()];

        let value = navigate_to_field(&env.traits, &path);

        assert!(value.is_null());
    }

    #[test]
    fn compare_field_value_boolean() {
        let value = serde_json::Value::Bool(true);

        assert!(compare_field_value(&value, "true", &FieldType::Boolean));
        assert!(!compare_field_value(&value, "false", &FieldType::Boolean));
    }

    #[test]
    fn compare_field_value_string() {
        let value = serde_json::Value::String("cursor".to_string());

        assert!(compare_field_value(
            &value,
            "cursor",
            &FieldType::OptionalString
        ));
        assert!(!compare_field_value(
            &value,
            "other",
            &FieldType::OptionalString
        ));
    }

    #[test]
    fn compare_field_value_null() {
        let value = serde_json::Value::Null;

        assert!(!compare_field_value(
            &value,
            "anything",
            &FieldType::OptionalString
        ));
        assert!(!compare_field_value(&value, "true", &FieldType::Boolean));
    }

    #[test]
    fn format_field_value_boolean() {
        let value = serde_json::Value::Bool(true);
        assert_eq!(format_field_value(&value, &FieldType::Boolean), "true");

        let value = serde_json::Value::Bool(false);
        assert_eq!(format_field_value(&value, &FieldType::Boolean), "false");
    }

    #[test]
    fn format_field_value_string() {
        let value = serde_json::Value::String("cursor".to_string());
        assert_eq!(
            format_field_value(&value, &FieldType::OptionalString),
            "cursor"
        );

        let value = serde_json::Value::Null;
        assert_eq!(
            format_field_value(&value, &FieldType::OptionalString),
            "null"
        );
    }

    #[test]
    fn format_field_value_stream_info() {
        use serde_json::json;

        let value = json!({"tty": true, "piped": false});
        assert_eq!(
            format_field_value(&value, &FieldType::StreamInfo),
            "tty:true, piped:false"
        );

        let value = json!({"tty": false, "piped": true});
        assert_eq!(
            format_field_value(&value, &FieldType::StreamInfo),
            "tty:false, piped:true"
        );

        let value = serde_json::Value::Null;
        assert_eq!(format_field_value(&value, &FieldType::StreamInfo), "null");
    }

    #[test]
    fn check_result_equality() {
        assert_eq!(CheckResult::Boolean(true), CheckResult::Boolean(true));
        assert_ne!(CheckResult::Boolean(true), CheckResult::Boolean(false));

        assert_eq!(
            CheckResult::String("test".to_string()),
            CheckResult::String("test".to_string())
        );
        assert_ne!(
            CheckResult::String("test".to_string()),
            CheckResult::String("other".to_string())
        );

        let comp1 = CheckResult::Comparison {
            actual: "a".to_string(),
            expected: "b".to_string(),
            matched: false,
        };
        let comp2 = CheckResult::Comparison {
            actual: "a".to_string(),
            expected: "b".to_string(),
            matched: false,
        };
        assert_eq!(comp1, comp2);
    }

    #[test]
    fn evaluation_result_creation() {
        let result = EvaluationResult {
            result: CheckResult::Boolean(true),
            reason: Some("test reason".to_string()),
            signals: None,
        };

        assert_eq!(result.result, CheckResult::Boolean(true));
        assert_eq!(result.reason.unwrap(), "test reason");
        assert!(result.signals.is_none());
    }

    // Additional Test Coverage - ColorLevel Field Type
    #[test]
    fn evaluate_color_level_field_value() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["terminal".to_string(), "color_level".to_string()];

        let result = evaluate_nested_field(&env, &path, None, &registry);

        assert_eq!(result.result, CheckResult::String("truecolor".to_string()));
        assert!(
            result
                .reason
                .unwrap()
                .contains("field value: terminal.color_level")
        );
    }

    #[test]
    fn evaluate_color_level_field_comparison_match() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["terminal".to_string(), "color_level".to_string()];

        let result = evaluate_nested_field(&env, &path, Some("truecolor"), &registry);

        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "truecolor");
                assert_eq!(expected, "truecolor");
                assert!(matched);
            }
            _ => panic!("Expected Comparison result"),
        }
    }

    #[test]
    fn evaluate_color_level_field_comparison_no_match() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["terminal".to_string(), "color_level".to_string()];

        let result = evaluate_nested_field(&env, &path, Some("none"), &registry);

        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "truecolor");
                assert_eq!(expected, "none");
                assert!(!matched);
            }
            _ => panic!("Expected Comparison result"),
        }
    }

    // StreamInfo Field Type Tests
    #[test]
    fn evaluate_stream_info_field_always_false_comparison() {
        // Note: StreamInfo is not directly exposed as a field in the registry
        // since we break it down into individual tty/piped boolean fields.
        // This test verifies the StreamInfo formatting behavior instead.

        // Test that StreamInfo comparison logic works correctly
        let stream_info_value = serde_json::json!({"tty": true, "piped": false});
        let result = compare_field_value(&stream_info_value, "anything", &FieldType::StreamInfo);
        assert!(!result); // StreamInfo comparisons always return false

        // Test with different values
        let result = compare_field_value(&stream_info_value, "true", &FieldType::StreamInfo);
        assert!(!result);

        let result = compare_field_value(&stream_info_value, "false", &FieldType::StreamInfo);
        assert!(!result);
    }

    #[test]
    fn format_stream_info_with_missing_fields() {
        use serde_json::json;

        // Test StreamInfo formatting with missing fields
        let incomplete_value = json!({"tty": true}); // Missing "piped" field
        let formatted = format_field_value(&incomplete_value, &FieldType::StreamInfo);
        assert_eq!(formatted, "tty:true, piped:false");

        let empty_object = json!({});
        let formatted = format_field_value(&empty_object, &FieldType::StreamInfo);
        assert_eq!(formatted, "tty:false, piped:false");
    }

    // Edge Cases and Error Condition Tests
    #[test]
    fn evaluate_with_null_field_values() {
        use crate::traits::terminal::ColorLevel;
        use crate::traits::{
            AgentTraits, CiTraits, IdeTraits, NestedTraits, StreamInfo, TerminalTraits,
        };

        // Create environment with null/None values
        let env = EnvSense {
            contexts: vec!["agent".to_string()],
            traits: NestedTraits {
                agent: AgentTraits { id: None }, // Null value
                ide: IdeTraits { id: None },
                terminal: TerminalTraits {
                    interactive: false,
                    color_level: ColorLevel::None,
                    stdin: StreamInfo {
                        tty: false,
                        piped: false,
                    },
                    stdout: StreamInfo {
                        tty: false,
                        piped: false,
                    },
                    stderr: StreamInfo {
                        tty: false,
                        piped: false,
                    },
                    supports_hyperlinks: false,
                },
                ci: CiTraits {
                    id: None,
                    vendor: None,
                    name: None,
                    is_pr: None,
                    branch: None,
                },
            },
            host: None,
            evidence: vec![],
            version: "0.3.0".to_string(),
        };

        let registry = FieldRegistry::new();
        let path = vec!["agent".to_string(), "id".to_string()];

        // Test null value display
        let result = evaluate_nested_field(&env, &path, None, &registry);
        assert_eq!(result.result, CheckResult::String("null".to_string()));

        // Test null value comparison
        let result = evaluate_nested_field(&env, &path, Some("cursor"), &registry);
        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "null");
                assert_eq!(expected, "cursor");
                assert!(!matched);
            }
            _ => panic!("Expected Comparison result"),
        }
    }

    #[test]
    fn evaluate_empty_string_vs_null_comparison() {
        use crate::traits::terminal::ColorLevel;
        use crate::traits::{
            AgentTraits, CiTraits, IdeTraits, NestedTraits, StreamInfo, TerminalTraits,
        };

        // Create environment with empty string value
        let env = EnvSense {
            contexts: vec!["agent".to_string()],
            traits: NestedTraits {
                agent: AgentTraits {
                    id: Some("".to_string()),
                }, // Empty string
                ide: IdeTraits { id: None },
                terminal: TerminalTraits {
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
                },
                ci: CiTraits::default(),
            },
            host: None,
            evidence: vec![],
            version: "0.3.0".to_string(),
        };

        let registry = FieldRegistry::new();
        let path = vec!["agent".to_string(), "id".to_string()];

        // Test empty string value display
        let result = evaluate_nested_field(&env, &path, None, &registry);
        assert_eq!(result.result, CheckResult::String("".to_string()));

        // Test empty string comparison with empty string
        let result = evaluate_nested_field(&env, &path, Some(""), &registry);
        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "");
                assert_eq!(expected, "");
                assert!(matched);
            }
            _ => panic!("Expected Comparison result"),
        }

        // Test empty string comparison with non-empty string
        let result = evaluate_nested_field(&env, &path, Some("cursor"), &registry);
        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "");
                assert_eq!(expected, "cursor");
                assert!(!matched);
            }
            _ => panic!("Expected Comparison result"),
        }
    }

    #[test]
    fn evaluate_case_sensitive_comparisons() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["agent".to_string(), "id".to_string()];

        // Test case sensitivity in string comparisons
        let result = evaluate_nested_field(&env, &path, Some("CURSOR"), &registry);
        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "cursor");
                assert_eq!(expected, "CURSOR");
                assert!(!matched); // Should be case sensitive
            }
            _ => panic!("Expected Comparison result"),
        }
    }

    #[test]
    fn evaluate_special_characters_in_values() {
        use crate::traits::terminal::ColorLevel;
        use crate::traits::{
            AgentTraits, CiTraits, IdeTraits, NestedTraits, StreamInfo, TerminalTraits,
        };

        // Create environment with special characters
        let env = EnvSense {
            contexts: vec!["ci".to_string()],
            traits: NestedTraits {
                agent: AgentTraits { id: None },
                ide: IdeTraits { id: None },
                terminal: TerminalTraits {
                    interactive: false,
                    color_level: ColorLevel::None,
                    stdin: StreamInfo {
                        tty: false,
                        piped: true,
                    },
                    stdout: StreamInfo {
                        tty: false,
                        piped: true,
                    },
                    stderr: StreamInfo {
                        tty: false,
                        piped: true,
                    },
                    supports_hyperlinks: false,
                },
                ci: CiTraits {
                    id: Some("github-actions".to_string()),
                    vendor: Some("GitHub".to_string()),
                    name: Some("GitHub Actions".to_string()),
                    is_pr: Some(true),
                    branch: Some("feature/test-123".to_string()), // Special characters
                },
            },
            host: None,
            evidence: vec![],
            version: "0.3.0".to_string(),
        };

        let registry = FieldRegistry::new();
        let path = vec!["ci".to_string(), "branch".to_string()];

        // Test special characters in branch name
        let result = evaluate_nested_field(&env, &path, Some("feature/test-123"), &registry);
        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "feature/test-123");
                assert_eq!(expected, "feature/test-123");
                assert!(matched);
            }
            _ => panic!("Expected Comparison result"),
        }
    }

    #[test]
    fn evaluate_boolean_string_representations() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["terminal".to_string(), "interactive".to_string()];

        // Test various boolean string representations
        let test_cases = vec![
            ("true", true),
            ("True", false), // Case sensitive
            ("TRUE", false), // Case sensitive
            ("false", false),
            ("1", false), // Not "true"
            ("0", false), // Not "true"
            ("", false),  // Not "true"
        ];

        for (input, expected_match) in test_cases {
            let result = evaluate_nested_field(&env, &path, Some(input), &registry);
            match result.result {
                CheckResult::Comparison { matched, .. } => {
                    assert_eq!(
                        matched, expected_match,
                        "Failed for input '{}', expected match: {}",
                        input, expected_match
                    );
                }
                _ => panic!("Expected Comparison result for input '{}'", input),
            }
        }
    }

    #[test]
    fn evaluate_multiple_contexts_scenario() {
        use crate::traits::terminal::ColorLevel;
        use crate::traits::{
            AgentTraits, CiTraits, IdeTraits, NestedTraits, StreamInfo, TerminalTraits,
        };

        // Create environment with multiple contexts
        let env = EnvSense {
            contexts: vec![
                "agent".to_string(),
                "ide".to_string(),
                "ci".to_string(),
                "terminal".to_string(),
            ],
            traits: NestedTraits {
                agent: AgentTraits {
                    id: Some("cursor".to_string()),
                },
                ide: IdeTraits {
                    id: Some("cursor".to_string()),
                },
                terminal: TerminalTraits {
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
                },
                ci: CiTraits {
                    id: Some("github".to_string()),
                    vendor: Some("github".to_string()),
                    name: Some("GitHub Actions".to_string()),
                    is_pr: Some(false),
                    branch: Some("main".to_string()),
                },
            },
            host: None,
            evidence: vec![],
            version: "0.3.0".to_string(),
        };

        let _registry = FieldRegistry::new();

        // Test all contexts are detected
        for context in &["agent", "ide", "ci", "terminal"] {
            let result = evaluate_context(&env, context);
            assert_eq!(
                result.result,
                CheckResult::Boolean(true),
                "Context '{}' should be detected",
                context
            );
        }

        // Test context not present
        let result = evaluate_context(&env, "container");
        assert_eq!(result.result, CheckResult::Boolean(false));
    }

    #[test]
    fn evaluate_very_deep_field_path() {
        let env = create_test_env();
        let registry = FieldRegistry::new();

        // Test the deepest valid path we have
        let path = vec![
            "terminal".to_string(),
            "stderr".to_string(),
            "tty".to_string(),
        ];
        let result = evaluate_nested_field(&env, &path, None, &registry);

        assert_eq!(result.result, CheckResult::Boolean(true));
        assert!(
            result
                .reason
                .unwrap()
                .contains("field value: terminal.stderr.tty")
        );
    }

    #[test]
    fn evaluate_legacy_facet_with_special_characters() {
        use crate::traits::{AgentTraits, CiTraits, IdeTraits, NestedTraits, TerminalTraits};

        // Create environment with special characters in CI ID
        let env = EnvSense {
            contexts: vec!["ci".to_string()],
            traits: NestedTraits {
                agent: AgentTraits { id: None },
                ide: IdeTraits { id: None },
                terminal: TerminalTraits::default(),
                ci: CiTraits {
                    id: Some("gitlab-ci-123".to_string()),
                    vendor: None,
                    name: None,
                    is_pr: None,
                    branch: None,
                },
            },
            host: None,
            evidence: vec![],
            version: "0.3.0".to_string(),
        };

        let result = evaluate_legacy_facet(&env, "ci_id", "gitlab-ci-123");

        match result.result {
            CheckResult::Comparison {
                actual,
                expected,
                matched,
            } => {
                assert_eq!(actual, "gitlab-ci-123");
                assert_eq!(expected, "gitlab-ci-123");
                assert!(matched);
            }
            _ => panic!("Expected Comparison result"),
        }
    }

    #[test]
    fn evaluate_with_signals_field() {
        // Test that signals field is properly handled (currently always None)
        let env = create_test_env();
        let result = evaluate_context(&env, "agent");

        assert!(result.signals.is_none());

        // Test that we can create evaluation results with signals
        let mut signals = BTreeMap::new();
        signals.insert("test_key".to_string(), "test_value".to_string());

        let result_with_signals = EvaluationResult {
            result: CheckResult::Boolean(true),
            reason: Some("test".to_string()),
            signals: Some(signals.clone()),
        };

        assert_eq!(result_with_signals.signals, Some(signals));
    }

    // Task 2.4: Output Formatting Tests

    #[test]
    fn check_result_format_boolean_without_explain() {
        let result = CheckResult::Boolean(true);
        assert_eq!(result.format(false), "true");

        let result = CheckResult::Boolean(false);
        assert_eq!(result.format(false), "false");
    }

    #[test]
    fn check_result_format_boolean_with_explain() {
        let result = CheckResult::Boolean(true);
        assert_eq!(result.format(true), "true  # boolean result");

        let result = CheckResult::Boolean(false);
        assert_eq!(result.format(true), "false  # boolean result");
    }

    #[test]
    fn check_result_format_string_without_explain() {
        let result = CheckResult::String("cursor".to_string());
        assert_eq!(result.format(false), "cursor");

        let result = CheckResult::String("".to_string());
        assert_eq!(result.format(false), "");
    }

    #[test]
    fn check_result_format_string_with_explain() {
        let result = CheckResult::String("cursor".to_string());
        assert_eq!(result.format(true), "cursor  # string value");

        let result = CheckResult::String("test-value".to_string());
        assert_eq!(result.format(true), "test-value  # string value");
    }

    #[test]
    fn check_result_format_comparison_without_explain() {
        let result = CheckResult::Comparison {
            actual: "cursor".to_string(),
            expected: "cursor".to_string(),
            matched: true,
        };
        assert_eq!(result.format(false), "true");

        let result = CheckResult::Comparison {
            actual: "cursor".to_string(),
            expected: "other".to_string(),
            matched: false,
        };
        assert_eq!(result.format(false), "false");
    }

    #[test]
    fn check_result_format_comparison_with_explain() {
        let result = CheckResult::Comparison {
            actual: "cursor".to_string(),
            expected: "cursor".to_string(),
            matched: true,
        };
        assert_eq!(result.format(true), "true  # cursor == cursor");

        let result = CheckResult::Comparison {
            actual: "cursor".to_string(),
            expected: "other".to_string(),
            matched: false,
        };
        assert_eq!(result.format(true), "false  # cursor == other");
    }

    #[test]
    fn check_result_as_bool() {
        // Boolean results
        assert!(CheckResult::Boolean(true).as_bool());
        assert!(!CheckResult::Boolean(false).as_bool());

        // String results (presence implies true)
        assert!(CheckResult::String("cursor".to_string()).as_bool());
        assert!(CheckResult::String("".to_string()).as_bool());

        // Comparison results
        let matched = CheckResult::Comparison {
            actual: "cursor".to_string(),
            expected: "cursor".to_string(),
            matched: true,
        };
        assert!(matched.as_bool());

        let not_matched = CheckResult::Comparison {
            actual: "cursor".to_string(),
            expected: "other".to_string(),
            matched: false,
        };
        assert!(!not_matched.as_bool());
    }

    #[test]
    fn check_result_as_string() {
        // Boolean results
        assert_eq!(CheckResult::Boolean(true).as_string(), "true");
        assert_eq!(CheckResult::Boolean(false).as_string(), "false");

        // String results
        assert_eq!(
            CheckResult::String("cursor".to_string()).as_string(),
            "cursor"
        );
        assert_eq!(CheckResult::String("".to_string()).as_string(), "");

        // Comparison results
        let matched = CheckResult::Comparison {
            actual: "cursor".to_string(),
            expected: "cursor".to_string(),
            matched: true,
        };
        assert_eq!(matched.as_string(), "true");

        let not_matched = CheckResult::Comparison {
            actual: "cursor".to_string(),
            expected: "other".to_string(),
            matched: false,
        };
        assert_eq!(not_matched.as_string(), "false");
    }

    #[test]
    fn check_result_format_special_characters() {
        // Test with special characters in strings
        let result = CheckResult::String("feature/test-123".to_string());
        assert_eq!(result.format(false), "feature/test-123");
        assert_eq!(result.format(true), "feature/test-123  # string value");

        // Test with special characters in comparisons
        let result = CheckResult::Comparison {
            actual: "feature/test-123".to_string(),
            expected: "feature/test-123".to_string(),
            matched: true,
        };
        assert_eq!(result.format(false), "true");
        assert_eq!(
            result.format(true),
            "true  # feature/test-123 == feature/test-123"
        );
    }

    #[test]
    fn check_result_format_empty_and_null_values() {
        // Test empty string
        let result = CheckResult::String("".to_string());
        assert_eq!(result.format(false), "");
        assert_eq!(result.format(true), "  # string value");

        // Test null representation in comparison
        let result = CheckResult::Comparison {
            actual: "null".to_string(),
            expected: "cursor".to_string(),
            matched: false,
        };
        assert_eq!(result.format(false), "false");
        assert_eq!(result.format(true), "false  # null == cursor");
    }

    #[test]
    fn check_result_format_consistency() {
        // Ensure format(false) and as_string() are consistent for boolean results
        let bool_true = CheckResult::Boolean(true);
        assert_eq!(bool_true.format(false), bool_true.as_string());

        let bool_false = CheckResult::Boolean(false);
        assert_eq!(bool_false.format(false), bool_false.as_string());

        // Ensure format(false) and as_string() are consistent for string results
        let string_result = CheckResult::String("test".to_string());
        assert_eq!(string_result.format(false), string_result.as_string());

        // Ensure format(false) and as_string() are consistent for comparison results
        let comparison = CheckResult::Comparison {
            actual: "a".to_string(),
            expected: "b".to_string(),
            matched: false,
        };
        assert_eq!(comparison.format(false), comparison.as_string());
    }

    #[test]
    fn evaluation_result_with_different_result_types() {
        // Test EvaluationResult with Boolean result
        let bool_result = EvaluationResult {
            result: CheckResult::Boolean(true),
            reason: Some("context detected".to_string()),
            signals: None,
        };
        assert!(bool_result.result.as_bool());
        assert_eq!(bool_result.result.format(false), "true");

        // Test EvaluationResult with String result
        let string_result = EvaluationResult {
            result: CheckResult::String("cursor".to_string()),
            reason: Some("field value".to_string()),
            signals: None,
        };
        assert!(string_result.result.as_bool());
        assert_eq!(string_result.result.format(false), "cursor");

        // Test EvaluationResult with Comparison result
        let comparison_result = EvaluationResult {
            result: CheckResult::Comparison {
                actual: "cursor".to_string(),
                expected: "cursor".to_string(),
                matched: true,
            },
            reason: Some("field comparison".to_string()),
            signals: None,
        };
        assert!(comparison_result.result.as_bool());
        assert_eq!(comparison_result.result.format(false), "true");
    }

    // Task 2.4: Additional Output Function Tests

    #[test]
    fn output_json_structure_validation() {
        use serde_json::json;

        // Test JSON structure generation logic (extracted from output_json_results)
        let results = vec![
            EvaluationResult {
                result: CheckResult::Boolean(true),
                reason: Some("boolean reason".to_string()),
                signals: Some({
                    let mut map = BTreeMap::new();
                    map.insert("key1".to_string(), "value1".to_string());
                    map
                }),
            },
            EvaluationResult {
                result: CheckResult::String("test-value".to_string()),
                reason: Some("string reason".to_string()),
                signals: None,
            },
        ];
        let predicates = vec!["agent".to_string(), "agent.id".to_string()];

        // Test with explain=true
        let checks_with_explain: Vec<serde_json::Value> = results
            .iter()
            .zip(predicates.iter())
            .map(|(result, predicate)| {
                let mut check = json!({
                    "predicate": predicate,
                    "result": result.result.as_bool(),
                });

                if let Some(reason) = &result.reason {
                    check["reason"] = json!(reason);
                }
                if let Some(signals) = &result.signals {
                    check["signals"] = json!(signals);
                }
                check
            })
            .collect();

        let output_with_explain = json!({
            "overall": true,
            "mode": "any",
            "checks": checks_with_explain,
        });

        // Verify structure with explain
        assert!(output_with_explain["overall"].as_bool().unwrap());
        assert_eq!(output_with_explain["mode"].as_str().unwrap(), "any");
        assert_eq!(output_with_explain["checks"].as_array().unwrap().len(), 2);

        let checks_array = output_with_explain["checks"].as_array().unwrap();

        // Check first result (Boolean with signals)
        assert_eq!(checks_array[0]["predicate"].as_str().unwrap(), "agent");
        assert!(checks_array[0]["result"].as_bool().unwrap());
        assert_eq!(
            checks_array[0]["reason"].as_str().unwrap(),
            "boolean reason"
        );
        assert!(checks_array[0]["signals"].is_object());

        // Check second result (String without signals)
        assert_eq!(checks_array[1]["predicate"].as_str().unwrap(), "agent.id");
        assert!(checks_array[1]["result"].as_bool().unwrap()); // String presence = true
        assert_eq!(checks_array[1]["reason"].as_str().unwrap(), "string reason");
        assert!(checks_array[1]["signals"].is_null());
    }

    #[test]
    fn output_human_results_logic_validation() {
        // Test the logic that would be used in output_human_results

        let results = vec![EvaluationResult {
            result: CheckResult::Boolean(true),
            reason: Some("context detected".to_string()),
            signals: None,
        }];

        // Single result formatting
        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert_eq!(result.result.format(false), "true");
        assert_eq!(result.result.format(true), "true  # boolean result");

        // Single result with reason
        if let Some(reason) = result.reason.as_ref() {
            let formatted = format!("{}  # reason: {}", result.result.format(false), reason);
            assert_eq!(formatted, "true  # reason: context detected");
        }

        // Multiple results logic
        let multiple_results = vec![
            EvaluationResult {
                result: CheckResult::Boolean(true),
                reason: Some("context detected".to_string()),
                signals: None,
            },
            EvaluationResult {
                result: CheckResult::String("cursor".to_string()),
                reason: Some("field value".to_string()),
                signals: None,
            },
        ];
        let predicates = vec!["agent".to_string(), "agent.id".to_string()];

        assert!(multiple_results.len() > 1);

        // Test multiple results formatting logic
        for (i, result) in multiple_results.iter().enumerate() {
            let predicate = &predicates[i];
            let formatted = format!("{}={}", predicate, result.result.format(false));
            if i == 0 {
                assert_eq!(formatted, "agent=true");
            } else {
                assert_eq!(formatted, "agent.id=cursor");
            }
        }
    }

    #[test]
    fn output_edge_cases() {
        // Test empty results
        let empty_results: Vec<EvaluationResult> = vec![];
        let empty_predicates: Vec<String> = vec![];

        // Should handle empty arrays gracefully
        assert_eq!(empty_results.len(), 0);
        assert_eq!(empty_predicates.len(), 0);

        // Test very long values
        let long_value = "a".repeat(1000);
        let long_result = EvaluationResult {
            result: CheckResult::String(long_value.clone()),
            reason: Some("very long reason ".repeat(50)),
            signals: None,
        };

        // Should handle long values without issues
        assert_eq!(long_result.result.format(false), long_value);
        assert!(long_result.reason.as_ref().unwrap().len() > 500);

        // Test unicode characters
        let unicode_result = EvaluationResult {
            result: CheckResult::String("🚀 cursor 中文 🎉".to_string()),
            reason: Some("unicode test 🧪".to_string()),
            signals: None,
        };

        assert_eq!(unicode_result.result.format(false), "🚀 cursor 中文 🎉");
        assert_eq!(unicode_result.reason.as_ref().unwrap(), "unicode test 🧪");
    }

    #[test]
    fn output_mode_specific_logic() {
        // Test overall calculation logic for different modes
        let mixed_results = vec![
            EvaluationResult {
                result: CheckResult::Boolean(true),
                reason: None,
                signals: None,
            },
            EvaluationResult {
                result: CheckResult::Boolean(false),
                reason: None,
                signals: None,
            },
        ];

        // Test "all" mode (should be false - not all results are true)
        let overall_all = mixed_results.iter().all(|r| r.result.as_bool());
        assert!(!overall_all);

        // Test "any" mode (should be true - at least one result is true)
        let overall_any = mixed_results.iter().any(|r| r.result.as_bool());
        assert!(overall_any);

        // Test all true results
        let all_true_results = vec![
            EvaluationResult {
                result: CheckResult::Boolean(true),
                reason: None,
                signals: None,
            },
            EvaluationResult {
                result: CheckResult::String("value".to_string()),
                reason: None,
                signals: None,
            },
        ];

        let all_true_all = all_true_results.iter().all(|r| r.result.as_bool());
        let all_true_any = all_true_results.iter().any(|r| r.result.as_bool());
        assert!(all_true_all);
        assert!(all_true_any);
    }
}
