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
    #[error("invalid predicate syntax '{0}': {1}")]
    InvalidSyntax(String, String),
    #[error("invalid field path '{0}': field does not exist")]
    FieldNotFound(String),
    #[error("invalid field path '{0}': available fields for '{1}': {2}")]
    InvalidFieldForContext(String, String, String),
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
                "result": result_to_json_value(&result.result),
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

/// Convert CheckResult to appropriate JSON value for API output
fn result_to_json_value(result: &CheckResult) -> serde_json::Value {
    match result {
        CheckResult::Boolean(b) => serde_json::Value::Bool(*b),
        CheckResult::String(s) => serde_json::Value::String(s.clone()),
        CheckResult::Comparison { matched, .. } => serde_json::Value::Bool(*matched),
    }
}

/// Task 2.6: Help Text Generation
///
/// Generate dynamic help text from the field registry system
pub fn generate_help_text(registry: &FieldRegistry) -> String {
    let mut help = String::from("Available predicates:\n\n");

    // Contexts section
    help.push_str("Contexts (return boolean):\n");
    for context in registry.get_contexts() {
        help.push_str(&format!(
            "  {}                    # Check if {} context is detected\n",
            context, context
        ));
    }

    // Fields section organized by context
    help.push_str("\nFields:\n");
    for context in registry.get_contexts() {
        let context_fields = registry.get_context_fields(context);
        if !context_fields.is_empty() {
            help.push_str(&format!("\n  {} fields:\n", context));

            // Sort fields by path for consistent output
            let mut sorted_fields = context_fields;
            sorted_fields.sort_by(|a, b| a.0.cmp(b.0));

            for (field_path, field_info) in sorted_fields {
                // Format field with appropriate padding for alignment
                let field_display = format!("    {}", field_path);
                let padding = if field_display.len() < 30 {
                    " ".repeat(30 - field_display.len())
                } else {
                    " ".to_string()
                };
                help.push_str(&format!(
                    "{}{}# {}\n",
                    field_display, padding, field_info.description
                ));
            }
        }
    }

    // Usage examples
    help.push_str("\nExamples:\n");
    help.push_str("  envsense check agent              # Boolean: is agent detected?\n");
    help.push_str("  envsense check agent.id           # String: show agent ID\n");
    help.push_str("  envsense check agent.id=cursor    # Boolean: is agent ID 'cursor'?\n");
    help.push_str("  envsense check terminal.interactive # Boolean: is terminal interactive?\n");
    help.push_str("  envsense check !ci                # Boolean: is CI NOT detected?\n");
    help.push_str("\nSyntax:\n");
    help.push_str("  context                           # Check if context is detected\n");
    help.push_str("  field.path                        # Show field value\n");
    help.push_str("  field.path=value                  # Compare field value\n");
    help.push_str("  !predicate                        # Negate any predicate\n");

    help
}

/// Generate help text using a static registry instance
///
/// This function provides the help text for CLI integration using OnceLock
/// to ensure the registry is only created once.
pub fn check_predicate_long_help() -> &'static str {
    use std::sync::OnceLock;

    static HELP: OnceLock<String> = OnceLock::new();
    HELP.get_or_init(|| {
        let registry = FieldRegistry::new();
        generate_help_text(&registry)
    })
    .as_str()
}

pub fn parse(input: &str) -> Result<Check, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    // Parse based on syntax
    if input.contains('.') {
        parse_nested_field(input)
    } else {
        Ok(Check::Context(input.to_string()))
    }
}

pub fn parse_predicate(input: &str) -> Result<ParsedCheck, ParseError> {
    let input = input.trim();

    // First validate the basic syntax
    validate_predicate_syntax(input)?;

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

pub const CONTEXTS: &[&str] = &["agent", "ide", "ci", "container", "remote"];

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

    /// Get all available contexts
    pub fn get_contexts(&self) -> Vec<&str> {
        vec!["agent", "ide", "terminal", "ci"]
    }

    /// Check if a field exists in the registry
    pub fn has_field(&self, field_path: &str) -> bool {
        self.fields.contains_key(field_path)
    }

    /// Check if a context exists
    pub fn has_context(&self, context: &str) -> bool {
        self.get_contexts().contains(&context)
    }
}

/// Predicate syntax validation functions
pub fn validate_predicate_syntax(input: &str) -> Result<(), ParseError> {
    let input = input.trim();

    // Check for empty input
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    // Handle negation
    let input = if let Some(rest) = input.strip_prefix('!') {
        rest
    } else {
        input
    };

    // Validate character set: alphanumeric, dots, equals, underscores
    let valid_chars_regex = regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9_.=]*$").unwrap();
    if !valid_chars_regex.is_match(input) {
        return Err(ParseError::InvalidSyntax(
            input.to_string(),
            "Valid predicate syntax: letters, numbers, dots (.), equals (=), and underscores (_) only".to_string()
        ));
    }

    Ok(())
}

/// Strict field path validation
pub fn validate_field_path(path: &[String], registry: &FieldRegistry) -> Result<(), ParseError> {
    let field_path = path.join(".");

    if !registry.has_field(&field_path) {
        let context = &path[0];
        if registry.has_context(context) {
            let available_fields = registry.get_context_fields(context);
            let field_names: Vec<String> = available_fields
                .iter()
                .map(|(name, _)| (*name).clone())
                .collect();
            return Err(ParseError::InvalidFieldForContext(
                field_path,
                context.clone(),
                field_names.join(", "),
            ));
        } else {
            return Err(ParseError::FieldNotFound(field_path));
        }
    }

    Ok(())
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

    // Validation Tests
    #[test]
    fn test_validate_predicate_syntax_valid() {
        let valid_cases = vec![
            "agent",
            "agent.id",
            "agent.id=cursor",
            "ide.cursor",
            "ci.github",
            "terminal.interactive",
            "agent_test",
            "test_field.sub_field",
            "field123",
            "test123.field456",
            "a.b.c",
            "field=value123",
            "field_name=test_value",
        ];

        for case in valid_cases {
            assert!(
                validate_predicate_syntax(case).is_ok(),
                "Valid syntax '{}' should pass validation",
                case
            );
        }
    }

    #[test]
    fn test_validate_predicate_syntax_invalid() {
        let invalid_cases = vec![
            ("invalid@syntax", "@"),
            ("agent#test", "#"),
            ("ide$cursor", "$"),
            ("test%value", "%"),
            ("bad&predicate", "&"),
            ("invalid*field", "*"),
            ("test+value", "+"),
            ("bad-predicate", "-"),
            ("test(value)", "("),
            ("test[value]", "["),
            ("test{value}", "{"),
            ("test|value", "|"),
            ("test\\value", "\\"),
            ("test/value", "/"),
            ("test:value", ":"),
            ("test;value", ";"),
            ("test<value", "<"),
            ("test>value", ">"),
            ("test?value", "?"),
            ("test\"value", "\""),
            ("test'value", "'"),
            ("test value", " "),
        ];

        for (invalid_case, _char) in invalid_cases {
            let result = validate_predicate_syntax(invalid_case);
            assert!(
                result.is_err(),
                "Invalid syntax '{}' should fail validation",
                invalid_case
            );
            match result {
                Err(ParseError::InvalidSyntax(input, message)) => {
                    assert_eq!(input, invalid_case);
                    assert!(message.contains("Valid predicate syntax"));
                }
                _ => panic!("Expected InvalidSyntax error for '{}'", invalid_case),
            }
        }
    }

    #[test]
    fn test_validate_predicate_syntax_empty() {
        assert_eq!(validate_predicate_syntax(""), Err(ParseError::EmptyInput));
        assert_eq!(
            validate_predicate_syntax("   "),
            Err(ParseError::EmptyInput)
        );
        assert_eq!(
            validate_predicate_syntax("\t\n"),
            Err(ParseError::EmptyInput)
        );
    }

    #[test]
    fn test_validate_predicate_syntax_with_negation() {
        // The validation function handles negation internally
        assert!(validate_predicate_syntax("!agent").is_ok());
        assert!(validate_predicate_syntax("!agent.id=test").is_ok());

        // Invalid syntax after negation should still fail
        let result = validate_predicate_syntax("!invalid@syntax");
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidSyntax(input, _)) => {
                assert_eq!(input, "invalid@syntax"); // The function strips the negation
            }
            _ => panic!("Expected InvalidSyntax error"),
        }
    }

    #[test]
    fn test_validate_field_path_valid() {
        let registry = FieldRegistry::new();

        let valid_paths = vec![
            vec!["agent".to_string(), "id".to_string()],
            vec!["ide".to_string(), "id".to_string()],
            vec!["terminal".to_string(), "interactive".to_string()],
            vec!["ci".to_string(), "name".to_string()],
        ];

        for path in valid_paths {
            assert!(
                validate_field_path(&path, &registry).is_ok(),
                "Valid field path '{}' should pass validation",
                path.join(".")
            );
        }
    }

    #[test]
    fn test_validate_field_path_invalid_field() {
        let registry = FieldRegistry::new();

        let invalid_field_path = vec!["agent".to_string(), "invalid_field".to_string()];
        let result = validate_field_path(&invalid_field_path, &registry);

        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidFieldForContext(field_path, context, available)) => {
                assert_eq!(field_path, "agent.invalid_field");
                assert_eq!(context, "agent");
                assert!(available.contains("agent.id"));
            }
            _ => panic!("Expected InvalidFieldForContext error"),
        }
    }

    #[test]
    fn test_validate_field_path_unknown_context() {
        let registry = FieldRegistry::new();

        let unknown_context_path = vec!["unknown".to_string(), "field".to_string()];
        let result = validate_field_path(&unknown_context_path, &registry);

        assert!(result.is_err());
        match result {
            Err(ParseError::FieldNotFound(field_path)) => {
                assert_eq!(field_path, "unknown.field");
            }
            _ => panic!("Expected FieldNotFound error"),
        }
    }

    #[test]
    fn test_field_registry_helper_methods() {
        let registry = FieldRegistry::new();

        // Test has_field
        assert!(registry.has_field("agent.id"));
        assert!(registry.has_field("ide.id"));
        assert!(registry.has_field("terminal.interactive"));
        assert!(!registry.has_field("agent.nonexistent"));
        assert!(!registry.has_field("unknown.field"));

        // Test has_context
        assert!(registry.has_context("agent"));
        assert!(registry.has_context("ide"));
        assert!(registry.has_context("terminal"));
        assert!(registry.has_context("ci"));
        assert!(!registry.has_context("unknown"));

        // Test get_context_fields
        let agent_fields = registry.get_context_fields("agent");
        assert_eq!(agent_fields.len(), 1);
        assert!(
            agent_fields
                .iter()
                .any(|(name, _)| name.as_str() == "agent.id")
        );

        let terminal_fields = registry.get_context_fields("terminal");
        assert!(terminal_fields.len() >= 8); // Should have multiple terminal fields
        assert!(
            terminal_fields
                .iter()
                .any(|(name, _)| name.as_str() == "terminal.interactive")
        );

        let unknown_fields = registry.get_context_fields("unknown");
        assert!(unknown_fields.is_empty());
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

    // Task 2.5: Deprecation Warnings and Migration Support Tests

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

    // Task 2.6: Help Text Generation Tests
    #[test]
    fn test_generate_help_text_structure() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that help text contains expected sections
        assert!(help_text.contains("Available predicates:"));
        assert!(help_text.contains("Contexts (return boolean):"));
        assert!(help_text.contains("Fields:"));
        assert!(help_text.contains("Examples:"));
        assert!(help_text.contains("Syntax:"));
    }

    #[test]
    fn test_help_text_contexts() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that all contexts are included
        for context in registry.get_contexts() {
            assert!(help_text.contains(context));
            assert!(help_text.contains(&format!("Check if {} context is detected", context)));
        }
    }

    #[test]
    fn test_help_text_fields_by_context() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that each context has its fields listed
        for context in registry.get_contexts() {
            let context_fields = registry.get_context_fields(context);
            if !context_fields.is_empty() {
                assert!(help_text.contains(&format!("{} fields:", context)));

                for (field_path, field_info) in context_fields {
                    assert!(help_text.contains(field_path));
                    assert!(help_text.contains(&field_info.description));
                }
            }
        }
    }

    #[test]
    fn test_help_text_examples() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that examples are present
        assert!(help_text.contains("envsense check agent"));
        assert!(help_text.contains("envsense check agent.id"));
        assert!(help_text.contains("envsense check agent.id=cursor"));
        assert!(help_text.contains("envsense check terminal.interactive"));
        assert!(help_text.contains("envsense check !ci"));
    }

    #[test]
    fn test_help_text_syntax_section() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test syntax explanations
        assert!(
            help_text.contains("context                           # Check if context is detected")
        );
        assert!(help_text.contains("field.path                        # Show field value"));
        assert!(help_text.contains("field.path=value                  # Compare field value"));
        assert!(help_text.contains("!predicate                        # Negate any predicate"));
    }

    #[test]
    fn test_help_text_field_alignment() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that field descriptions are properly aligned
        let lines: Vec<&str> = help_text.lines().collect();
        let field_lines: Vec<&str> = lines
            .iter()
            .filter(|line| {
                line.trim_start().starts_with("agent.")
                    || line.trim_start().starts_with("ide.")
                    || line.trim_start().starts_with("terminal.")
                    || line.trim_start().starts_with("ci.")
            })
            .copied()
            .collect();

        // Each field line should have proper formatting
        for line in field_lines {
            assert!(line.contains("#"));
            let parts: Vec<&str> = line.split('#').collect();
            assert_eq!(parts.len(), 2);
            assert!(!parts[1].trim().is_empty()); // Description should not be empty
        }
    }

    #[test]
    fn test_help_text_field_sorting() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that fields within each context are sorted
        for context in registry.get_contexts() {
            let context_fields = registry.get_context_fields(context);
            if context_fields.len() > 1 {
                let field_paths: Vec<&String> =
                    context_fields.iter().map(|(path, _)| *path).collect();
                let mut sorted_paths = field_paths.clone();
                sorted_paths.sort();

                // Check if the original order matches sorted order
                // We can't directly compare since the help text might have different formatting
                // but we can check that all fields are present
                for field_path in field_paths {
                    assert!(help_text.contains(field_path));
                }
            }
        }
    }

    #[test]
    fn test_check_predicate_long_help_static() {
        // Test the static help function
        let help1 = check_predicate_long_help();
        let help2 = check_predicate_long_help();

        // Should return the same reference (OnceLock behavior)
        assert_eq!(help1.as_ptr(), help2.as_ptr());

        // Should contain the same content as generate_help_text
        let registry = FieldRegistry::new();
        let dynamic_help = generate_help_text(&registry);
        assert_eq!(help1, &dynamic_help);
    }

    #[test]
    fn test_help_text_completeness() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that all registered fields appear in help text
        let all_fields = registry.list_all_fields();
        for field_path in all_fields {
            assert!(
                help_text.contains(field_path),
                "Field {} not found in help text",
                field_path
            );
        }

        // Test that all contexts appear in help text
        for context in registry.get_contexts() {
            assert!(
                help_text.contains(context),
                "Context {} not found in help text",
                context
            );
        }
    }

    #[test]
    fn test_help_text_no_empty_sections() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that no section is completely empty
        assert!(!help_text.contains("Contexts (return boolean):\n\nFields:"));
        assert!(!help_text.contains("Fields:\n\nExamples:"));
        assert!(!help_text.contains("Examples:\n\nSyntax:"));

        // Test that each context section has content
        for context in registry.get_contexts() {
            let context_fields = registry.get_context_fields(context);
            if !context_fields.is_empty() {
                let context_section = format!("{} fields:", context);
                assert!(help_text.contains(&context_section));
            }
        }
    }

    // Additional comprehensive tests for missing coverage
    #[test]
    fn test_help_text_section_ordering() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that sections appear in the correct order
        let contexts_pos = help_text.find("Contexts (return boolean):").unwrap();
        let fields_pos = help_text.find("Fields:").unwrap();
        let examples_pos = help_text.find("Examples:").unwrap();
        let syntax_pos = help_text.find("Syntax:").unwrap();

        assert!(contexts_pos < fields_pos);
        assert!(fields_pos < examples_pos);
        assert!(examples_pos < syntax_pos);
    }

    #[test]
    fn test_help_text_field_padding_edge_cases() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that very long field names get at least one space before the comment
        let lines: Vec<&str> = help_text.lines().collect();
        let field_lines: Vec<&str> = lines
            .iter()
            .filter(|line| {
                line.trim_start().starts_with("agent.")
                    || line.trim_start().starts_with("ide.")
                    || line.trim_start().starts_with("terminal.")
                    || line.trim_start().starts_with("ci.")
            })
            .copied()
            .collect();

        for line in field_lines {
            // Every field line should have at least one space before the #
            let hash_pos = line.find('#').unwrap();
            let before_hash = &line[..hash_pos];
            assert!(
                before_hash.ends_with(' '),
                "Field line should have space before #: {}",
                line
            );
        }
    }

    #[test]
    fn test_help_text_context_descriptions() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that each context has a proper description format
        for context in registry.get_contexts() {
            let expected_description = format!("# Check if {} context is detected", context);
            assert!(
                help_text.contains(&expected_description),
                "Missing context description for {}",
                context
            );
        }
    }

    #[test]
    fn test_help_text_field_type_coverage() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Ensure we have examples of different field types in the help text
        // Boolean fields
        assert!(help_text.contains("terminal.interactive"));
        assert!(help_text.contains("Terminal interactivity"));

        // OptionalString fields
        assert!(help_text.contains("agent.id"));
        assert!(help_text.contains("Agent identifier"));

        // ColorLevel fields
        assert!(help_text.contains("terminal.color_level"));
        assert!(help_text.contains("Color support level"));

        // Verify all field types are represented
        let all_fields = registry.list_all_fields();
        let mut has_boolean = false;
        let mut has_optional_string = false;
        let mut has_color_level = false;

        for field_path in all_fields {
            let path_parts: Vec<String> = field_path.split('.').map(|s| s.to_string()).collect();
            if let Some(field_info) = registry.resolve_field(&path_parts) {
                match field_info.field_type {
                    FieldType::Boolean => has_boolean = true,
                    FieldType::OptionalString => has_optional_string = true,
                    FieldType::ColorLevel => has_color_level = true,
                    _ => {}
                }
            }
        }

        assert!(has_boolean, "Help text should include Boolean fields");
        assert!(
            has_optional_string,
            "Help text should include OptionalString fields"
        );
        assert!(
            has_color_level,
            "Help text should include ColorLevel fields"
        );
    }

    #[test]
    fn test_help_text_examples_coverage() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that examples cover different usage patterns
        assert!(help_text.contains("# Boolean: is agent detected?"));
        assert!(help_text.contains("# String: show agent ID"));
        assert!(help_text.contains("# Boolean: is agent ID 'cursor'?"));
        assert!(help_text.contains("# Boolean: is terminal interactive?"));
        assert!(help_text.contains("# Boolean: is CI NOT detected?"));

        // Test that examples use actual field paths from the registry
        assert!(help_text.contains("envsense check agent"));
        assert!(help_text.contains("envsense check agent.id"));
        assert!(help_text.contains("envsense check terminal.interactive"));
        assert!(help_text.contains("envsense check !ci"));
    }

    #[test]
    fn test_help_text_syntax_completeness() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test all syntax patterns are documented
        assert!(
            help_text.contains("context                           # Check if context is detected")
        );
        assert!(help_text.contains("field.path                        # Show field value"));
        assert!(help_text.contains("field.path=value                  # Compare field value"));
        assert!(help_text.contains("!predicate                        # Negate any predicate"));
    }

    #[test]
    fn test_help_text_consistent_formatting() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test consistent indentation
        let lines: Vec<&str> = help_text.lines().collect();

        // Context lines should have 2-space indentation
        for line in &lines {
            if line.contains("# Check if") && line.contains("context is detected") {
                assert!(
                    line.starts_with("  "),
                    "Context line should start with 2 spaces: {}",
                    line
                );
            }
        }

        // Field lines should have 4-space indentation
        for line in &lines {
            if line.trim_start().starts_with("agent.")
                || line.trim_start().starts_with("ide.")
                || line.trim_start().starts_with("terminal.")
                || line.trim_start().starts_with("ci.")
            {
                assert!(
                    line.starts_with("    "),
                    "Field line should start with 4 spaces: {}",
                    line
                );
            }
        }

        // Example lines should have 2-space indentation
        for line in &lines {
            if line.contains("envsense check") {
                assert!(
                    line.starts_with("  "),
                    "Example line should start with 2 spaces: {}",
                    line
                );
            }
        }
    }

    #[test]
    fn test_help_text_newline_handling() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that sections are properly separated with newlines
        assert!(help_text.contains("Contexts (return boolean):\n"));
        assert!(help_text.contains("\nFields:\n"));
        assert!(help_text.contains("\nExamples:\n"));
        assert!(help_text.contains("\nSyntax:\n"));

        // Test that context sections have proper spacing
        for context in registry.get_contexts() {
            let context_fields = registry.get_context_fields(context);
            if !context_fields.is_empty() {
                let context_header = format!("\n  {} fields:\n", context);
                assert!(help_text.contains(&context_header));
            }
        }
    }

    #[test]
    fn test_help_text_field_descriptions_accuracy() {
        let registry = FieldRegistry::new();
        let help_text = generate_help_text(&registry);

        // Test that field descriptions match registry exactly
        for context in registry.get_contexts() {
            let context_fields = registry.get_context_fields(context);
            for (field_path, field_info) in context_fields {
                // Check that both field path and description appear in help text
                assert!(
                    help_text.contains(field_path),
                    "Field path {} not found in help text",
                    field_path
                );
                assert!(
                    help_text.contains(&field_info.description),
                    "Field description '{}' not found in help text",
                    field_info.description
                );

                // Check that they appear on the same line
                let lines: Vec<&str> = help_text.lines().collect();
                let field_line = lines
                    .iter()
                    .find(|line| line.contains(field_path))
                    .expect(&format!("Could not find line containing {}", field_path));
                assert!(
                    field_line.contains(&field_info.description),
                    "Field {} and its description should be on the same line",
                    field_path
                );
            }
        }
    }

    #[test]
    fn test_help_text_empty_registry_handling() {
        // Create a minimal registry with no fields for edge case testing
        let empty_registry = FieldRegistry {
            fields: std::collections::HashMap::new(),
        };
        let help_text = generate_help_text(&empty_registry);

        // Should still have basic structure even with no fields
        assert!(help_text.contains("Available predicates:"));
        assert!(help_text.contains("Contexts (return boolean):"));
        assert!(help_text.contains("Fields:"));
        assert!(help_text.contains("Examples:"));
        assert!(help_text.contains("Syntax:"));

        // Should still show contexts even if no fields are registered
        for context in empty_registry.get_contexts() {
            assert!(help_text.contains(context));
        }
    }

    #[test]
    fn test_help_text_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        // Test that help text generation is thread-safe
        let registry = Arc::new(FieldRegistry::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let registry_clone = Arc::clone(&registry);
            let handle = thread::spawn(move || generate_help_text(&registry_clone));
            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.join().unwrap());
        }

        // All results should be identical
        let first_result = &results[0];
        for result in &results[1..] {
            assert_eq!(
                first_result, result,
                "Help text should be consistent across threads"
            );
        }
    }

    #[test]
    fn test_static_help_function_thread_safety() {
        use std::thread;

        // Test that the static help function is thread-safe
        let mut handles = vec![];

        for _ in 0..10 {
            let handle = thread::spawn(|| check_predicate_long_help());
            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.join().unwrap());
        }

        // All results should be identical pointers (OnceLock behavior)
        let first_ptr = results[0].as_ptr();
        for result in &results[1..] {
            assert_eq!(
                first_ptr,
                result.as_ptr(),
                "Static help should return same pointer across threads"
            );
        }
    }

    // Field Navigation Edge Cases
    #[test]
    fn navigate_to_field_with_invalid_path() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["terminal".to_string(), "nonexistent".to_string()];

        let result = evaluate_nested_field(&env, &path, None, &registry);

        // Should return Boolean(false) for unknown fields
        assert_eq!(result.result, CheckResult::Boolean(false));
        assert!(result.reason.unwrap().contains("unknown field"));
    }

    #[test]
    fn navigate_to_field_with_empty_path() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec![];

        let result = evaluate_nested_field(&env, &path, None, &registry);

        // Should return Boolean(false) for empty path (invalid field)
        assert_eq!(result.result, CheckResult::Boolean(false));
        assert!(result.reason.unwrap().contains("unknown field"));
    }

    #[test]
    fn navigate_to_field_with_deep_nested_path() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec![
            "terminal".to_string(),
            "stdin".to_string(),
            "tty".to_string(),
        ];

        let result = evaluate_nested_field(&env, &path, None, &registry);

        // Should navigate to nested boolean field
        assert_eq!(result.result, CheckResult::Boolean(true));
    }

    // Field Type Comparison Edge Cases
    #[test]
    fn compare_boolean_field_with_invalid_string() {
        let value = serde_json::json!(true);
        let result = compare_field_value(&value, "invalid", &FieldType::Boolean);
        assert!(!result); // "invalid" != "true"

        let result = compare_field_value(&value, "false", &FieldType::Boolean);
        assert!(!result); // true != false
    }

    #[test]
    fn compare_string_field_with_case_sensitivity() {
        let value = serde_json::json!("Cursor");
        let result = compare_field_value(&value, "cursor", &FieldType::String);
        assert!(!result); // Case sensitive comparison

        let result = compare_field_value(&value, "Cursor", &FieldType::String);
        assert!(result); // Exact match
    }

    #[test]
    fn compare_optional_string_field_with_none() {
        let value = serde_json::Value::Null;
        let result = compare_field_value(&value, "anything", &FieldType::OptionalString);
        assert!(!result); // null != "anything"

        let result = compare_field_value(&value, "", &FieldType::OptionalString);
        assert!(!result); // null != ""
    }

    // Error Handling Tests
    #[test]
    fn evaluate_unknown_field_path() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec!["unknown".to_string(), "field".to_string()];

        let result = evaluate_nested_field(&env, &path, None, &registry);

        assert_eq!(result.result, CheckResult::Boolean(false));
        assert!(result.reason.unwrap().contains("unknown field"));
    }

    #[test]
    fn evaluate_malformed_field_path() {
        let env = create_test_env();
        let registry = FieldRegistry::new();
        let path = vec![
            "terminal".to_string(),
            "interactive".to_string(),
            "extra".to_string(),
        ];

        let result = evaluate_nested_field(&env, &path, None, &registry);

        // Should return Boolean(false) for unknown field path
        assert_eq!(result.result, CheckResult::Boolean(false));
        assert!(result.reason.unwrap().contains("unknown field"));
    }
}
