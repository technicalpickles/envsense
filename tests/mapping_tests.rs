use envsense::detectors::env_mapping::{
    EnvIndicator, EnvMapping, get_agent_mappings, get_ci_mappings, get_ide_mappings,
};
use std::collections::HashMap;

fn create_env_vars(vars: Vec<(&str, &str)>) -> HashMap<String, String> {
    let mut env_map = HashMap::new();
    for (k, v) in vars {
        env_map.insert(k.to_string(), v.to_string());
    }
    env_map
}

#[test]
fn test_env_indicator_exact_match() {
    let indicator = EnvIndicator {
        key: "TEST_VAR".to_string(),
        value: Some("expected_value".to_string()),
        required: false,
        prefix: false,
        contains: None,
        priority: 0,
    };

    let env_vars = create_env_vars(vec![("TEST_VAR", "expected_value")]);
    let mapping = EnvMapping {
        id: "test".to_string(),
        confidence: 1.0,
        indicators: vec![indicator],
        facets: HashMap::new(),
        contexts: vec![],
        value_mappings: vec![],
    };

    assert!(mapping.matches(&env_vars));
}

#[test]
fn test_env_indicator_presence_only() {
    let indicator = EnvIndicator {
        key: "TEST_VAR".to_string(),
        value: None,
        required: false,
        prefix: false,
        contains: None,
        priority: 0,
    };

    let env_vars = create_env_vars(vec![("TEST_VAR", "any_value")]);
    let mapping = EnvMapping {
        id: "test".to_string(),
        confidence: 1.0,
        indicators: vec![indicator],
        facets: HashMap::new(),
        contexts: vec![],
        value_mappings: vec![],
    };

    assert!(mapping.matches(&env_vars));
}

#[test]
fn test_env_indicator_contains_match() {
    let indicator = EnvIndicator {
        key: "VERSION".to_string(),
        value: None,
        required: false,
        prefix: false,
        contains: Some("insider".to_string()),
        priority: 0,
    };

    let env_vars = create_env_vars(vec![("VERSION", "1.85.0-insider")]);
    let mapping = EnvMapping {
        id: "test".to_string(),
        confidence: 1.0,
        indicators: vec![indicator],
        facets: HashMap::new(),
        contexts: vec![],
        value_mappings: vec![],
    };

    assert!(mapping.matches(&env_vars));
}

#[test]
fn test_env_indicator_contains_no_match() {
    let indicator = EnvIndicator {
        key: "VERSION".to_string(),
        value: None,
        required: false,
        prefix: false,
        contains: Some("insider".to_string()),
        priority: 0,
    };

    let env_vars = create_env_vars(vec![("VERSION", "1.85.0")]);
    let mapping = EnvMapping {
        id: "test".to_string(),
        confidence: 1.0,
        indicators: vec![indicator],
        facets: HashMap::new(),
        contexts: vec![],
        value_mappings: vec![],
    };

    assert!(!mapping.matches(&env_vars));
}

#[test]
fn test_env_indicator_contains_case_insensitive() {
    let indicator = EnvIndicator {
        key: "VERSION".to_string(),
        value: None,
        required: false,
        prefix: false,
        contains: Some("INSIDER".to_string()),
        priority: 0,
    };

    let env_vars = create_env_vars(vec![("VERSION", "1.85.0-insider")]);
    let mapping = EnvMapping {
        id: "test".to_string(),
        confidence: 1.0,
        indicators: vec![indicator],
        facets: HashMap::new(),
        contexts: vec![],
        value_mappings: vec![],
    };

    assert!(mapping.matches(&env_vars));
}

#[test]
fn test_env_indicator_prefix_match() {
    let indicator = EnvIndicator {
        key: "TEST_".to_string(),
        value: None,
        required: false,
        prefix: true,
        contains: None,
        priority: 0,
    };

    let env_vars = create_env_vars(vec![
        ("TEST_VAR1", "value1"),
        ("TEST_VAR2", "value2"),
        ("OTHER_VAR", "value3"),
    ]);
    let mapping = EnvMapping {
        id: "test".to_string(),
        confidence: 1.0,
        indicators: vec![indicator],
        facets: HashMap::new(),
        contexts: vec![],
        value_mappings: vec![],
    };

    assert!(mapping.matches(&env_vars));
}

#[test]
fn test_env_indicator_required_and_optional() {
    let required_indicator = EnvIndicator {
        key: "REQUIRED_VAR".to_string(),
        value: Some("required_value".to_string()),
        required: true,
        prefix: false,
        contains: None,
        priority: 0,
    };

    let optional_indicator = EnvIndicator {
        key: "OPTIONAL_VAR".to_string(),
        value: None,
        required: false,
        prefix: false,
        contains: None,
        priority: 0,
    };

    let env_vars = create_env_vars(vec![
        ("REQUIRED_VAR", "required_value"),
        ("OPTIONAL_VAR", "optional_value"),
    ]);
    let mapping = EnvMapping {
        id: "test".to_string(),
        confidence: 1.0,
        indicators: vec![required_indicator, optional_indicator],
        facets: HashMap::new(),
        contexts: vec![],
        value_mappings: vec![],
    };

    assert!(mapping.matches(&env_vars));
}

#[test]
fn test_env_indicator_required_missing() {
    let required_indicator = EnvIndicator {
        key: "REQUIRED_VAR".to_string(),
        value: Some("required_value".to_string()),
        required: true,
        prefix: false,
        contains: None,
        priority: 0,
    };

    let optional_indicator = EnvIndicator {
        key: "OPTIONAL_VAR".to_string(),
        value: None,
        required: false,
        prefix: false,
        contains: None,
        priority: 0,
    };

    let env_vars = create_env_vars(vec![("OPTIONAL_VAR", "optional_value")]);
    let mapping = EnvMapping {
        id: "test".to_string(),
        confidence: 1.0,
        indicators: vec![required_indicator, optional_indicator],
        facets: HashMap::new(),
        contexts: vec![],
        value_mappings: vec![],
    };

    assert!(!mapping.matches(&env_vars));
}

#[test]
fn test_get_highest_priority() {
    let mapping = EnvMapping {
        id: "test".to_string(),
        confidence: 1.0,
        indicators: vec![
            EnvIndicator {
                key: "VAR1".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 1,
            },
            EnvIndicator {
                key: "VAR2".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 3,
            },
            EnvIndicator {
                key: "VAR3".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 2,
            },
        ],
        facets: HashMap::new(),
        contexts: vec![],
        value_mappings: vec![],
    };

    assert_eq!(mapping.get_highest_priority(), 3);
}

#[test]
fn test_ide_mapping_priority_ordering() {
    let mappings = get_ide_mappings();

    // Find Cursor mapping
    let cursor_mapping = mappings.iter().find(|m| m.id == "cursor-ide").unwrap();
    assert_eq!(cursor_mapping.get_highest_priority(), 3);

    // Find VS Code Insiders mapping
    let insiders_mapping = mappings.iter().find(|m| m.id == "vscode-insiders").unwrap();
    assert_eq!(insiders_mapping.get_highest_priority(), 2);

    // Find VS Code mapping
    let vscode_mapping = mappings.iter().find(|m| m.id == "vscode").unwrap();
    assert_eq!(vscode_mapping.get_highest_priority(), 1);
}

#[test]
fn test_cursor_mapping_matches() {
    let mappings = get_ide_mappings();
    let cursor_mapping = mappings.iter().find(|m| m.id == "cursor-ide").unwrap();

    let env_vars = create_env_vars(vec![
        ("TERM_PROGRAM", "vscode"),
        ("CURSOR_TRACE_ID", "abc123"),
    ]);

    assert!(cursor_mapping.matches(&env_vars));
}

#[test]
fn test_cursor_mapping_no_match() {
    let mappings = get_ide_mappings();
    let cursor_mapping = mappings.iter().find(|m| m.id == "cursor-ide").unwrap();

    let env_vars = create_env_vars(vec![
        ("TERM_PROGRAM", "vscode"),
        // Missing CURSOR_TRACE_ID
    ]);

    assert!(!cursor_mapping.matches(&env_vars));
}

#[test]
fn test_vscode_insiders_mapping_matches() {
    let mappings = get_ide_mappings();
    let insiders_mapping = mappings.iter().find(|m| m.id == "vscode-insiders").unwrap();

    let env_vars = create_env_vars(vec![
        ("TERM_PROGRAM", "vscode"),
        ("TERM_PROGRAM_VERSION", "1.85.0-insider"),
    ]);

    assert!(insiders_mapping.matches(&env_vars));
}

#[test]
fn test_vscode_insiders_mapping_no_match() {
    let mappings = get_ide_mappings();
    let insiders_mapping = mappings.iter().find(|m| m.id == "vscode-insiders").unwrap();

    let env_vars = create_env_vars(vec![
        ("TERM_PROGRAM", "vscode"),
        ("TERM_PROGRAM_VERSION", "1.85.0"), // No "insider"
    ]);

    assert!(!insiders_mapping.matches(&env_vars));
}

#[test]
fn test_vscode_mapping_matches() {
    let mappings = get_ide_mappings();
    let vscode_mapping = mappings.iter().find(|m| m.id == "vscode").unwrap();

    let env_vars = create_env_vars(vec![("TERM_PROGRAM", "vscode")]);

    assert!(vscode_mapping.matches(&env_vars));
}

#[test]
fn test_agent_mappings() {
    let mappings = get_agent_mappings();

    // Test Cursor agent mapping
    let cursor_mapping = mappings.iter().find(|m| m.id == "cursor").unwrap();
    let env_vars = create_env_vars(vec![("CURSOR_AGENT", "1")]);
    assert!(cursor_mapping.matches(&env_vars));

    // Test Replit agent mapping
    let replit_mapping = mappings.iter().find(|m| m.id == "replit-agent").unwrap();
    let env_vars = create_env_vars(vec![("REPL_ID", "abc123")]);
    assert!(replit_mapping.matches(&env_vars));
}

#[test]
fn test_ci_mappings() {
    let mappings = get_ci_mappings();

    // Test GitHub Actions mapping
    let github_mapping = mappings.iter().find(|m| m.id == "github-actions").unwrap();
    let env_vars = create_env_vars(vec![("GITHUB_ACTIONS", "true")]);
    assert!(github_mapping.matches(&env_vars));

    // Test GitLab CI mapping
    let gitlab_mapping = mappings.iter().find(|m| m.id == "gitlab-ci").unwrap();
    let env_vars = create_env_vars(vec![("GITLAB_CI", "true")]);
    assert!(gitlab_mapping.matches(&env_vars));
}

#[test]
fn test_mapping_evidence_generation() {
    let mapping = EnvMapping {
        id: "test".to_string(),
        confidence: 1.0,
        indicators: vec![
            EnvIndicator {
                key: "VAR1".to_string(),
                value: Some("value1".to_string()),
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            },
            EnvIndicator {
                key: "VAR2".to_string(),
                value: None,
                required: false,
                prefix: false,
                contains: None,
                priority: 0,
            },
        ],
        facets: HashMap::from([("test_facet".to_string(), "test_value".to_string())]),
        contexts: vec!["test_context".to_string()],
        value_mappings: vec![],
    };

    let env_vars = create_env_vars(vec![("VAR1", "value1"), ("VAR2", "value2")]);

    let evidence = mapping.get_evidence(&env_vars);
    assert_eq!(evidence.len(), 2);

    // Check that evidence contains both variables
    let keys: Vec<String> = evidence.iter().map(|(k, _)| k.clone()).collect();
    assert!(keys.contains(&"VAR1".to_string()));
    assert!(keys.contains(&"VAR2".to_string()));
}
