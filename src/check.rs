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
}
