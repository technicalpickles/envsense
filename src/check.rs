use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Check {
    Context(String),
    Facet { key: String, value: String },
    Trait { key: String },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParsedCheck {
    pub check: Check,
    pub negated: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("invalid check expression")]
    Invalid,
}

pub fn parse(input: &str) -> Result<Check, ParseError> {
    if let Some(rest) = input.strip_prefix("facet:") {
        let mut parts = rest.splitn(2, '=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next())
            && !key.is_empty()
            && !value.is_empty()
        {
            return Ok(Check::Facet {
                key: key.to_string(),
                value: value.to_string(),
            });
        }
        return Err(ParseError::Invalid);
    }
    if let Some(rest) = input.strip_prefix("trait:") {
        if !rest.is_empty() {
            return Ok(Check::Trait {
                key: rest.to_string(),
            });
        }
        return Err(ParseError::Invalid);
    }
    if input.is_empty() || input.contains(':') {
        return Err(ParseError::Invalid);
    }
    Ok(Check::Context(input.to_string()))
}

pub fn parse_predicate(input: &str) -> Result<ParsedCheck, ParseError> {
    let negated = input.starts_with('!');
    let expr = if negated { &input[1..] } else { input };
    let check = parse(expr)?;
    Ok(ParsedCheck { check, negated })
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

    #[test]
    fn parse_context() {
        assert_eq!(parse("agent"), Ok(Check::Context("agent".into())));
    }

    #[test]
    fn parse_facet() {
        assert_eq!(
            parse("facet:ide_id=vscode"),
            Ok(Check::Facet {
                key: "ide_id".into(),
                value: "vscode".into()
            })
        );
    }

    #[test]
    fn parse_trait() {
        assert_eq!(
            parse("trait:is_interactive"),
            Ok(Check::Trait {
                key: "is_interactive".into()
            })
        );
    }

    #[test]
    fn parse_invalid() {
        assert!(parse("facet:").is_err());
    }

    #[test]
    fn parse_not() {
        let pc = parse_predicate("!ci").unwrap();
        assert!(pc.negated);
        assert_eq!(pc.check, Check::Context("ci".into()));
    }
}
