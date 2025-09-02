use std::collections::HashMap;
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
}
