# CLI Streamlining Implementation - Phase 4: CLI Integration

## Overview

Phase 4 focuses on updating the CLI output rendering and user interface to work
with the new nested schema structure. This phase implements the enhanced output
formatting that supports different result types and provides a more intuitive
user experience.

## Objective

Update CLI output rendering and user interface for new schema, implementing:

- Enhanced output formatting for different result types (boolean, string,
  comparison)
- Context vs field output differentiation
- Nested trait rendering
- Updated help text and list commands

## Prerequisites

Before starting Phase 4, ensure the following phases are completed:

- **Phase 1**: New schema structures (`NestedTraits`, `AgentTraits`, etc.)
- **Phase 2**: Parser and evaluation logic with `FieldRegistry`
- **Phase 3**: Detection system populating nested structures

## Implementation Tasks

### Task 4.1: Update JSON Output Structure

**Files**: `src/main.rs`

**Objective**: Modify the `collect_snapshot()` function to work with the new
nested schema structure.

**Current Implementation**: The existing function uses flat facets and traits.

**Required Changes**:

```rust
fn collect_snapshot() -> Snapshot {
    let env = EnvSense::detect();

    Snapshot {
        contexts: env.contexts,  // Now Vec<String> instead of Contexts struct
        traits: serde_json::to_value(env.traits).unwrap(),  // Nested structure
        facets: json!({}),  // Empty for new schema (backward compatibility)
        meta: json!({
            "schema_version": env.version,
        }),
        evidence: serde_json::to_value(env.evidence).unwrap(),
    }
}
```

**Key Changes**:

- `contexts` is now a `Vec<String>` instead of a `Contexts` struct
- `traits` contains the nested `NestedTraits` structure
- `facets` is empty (maintained for backward compatibility)
- Schema version reflects the new version (0.3.0)

### Task 4.2: Update Human-Readable Output Rendering

**Files**: `src/main.rs`

**Objective**: Update the `render_human()` function to display nested traits in
a clear, hierarchical format.

**Required Changes**:

#### 4.2.1: Update Main Render Function

```rust
fn render_human(
    snapshot: &Snapshot,
    fields: Option<&str>,
    color: bool,
    raw: bool,
) -> Result<String, String> {
    let default_fields = ["contexts", "traits"];
    let selected: Vec<&str> = match fields {
        Some(f) => f.split(',').map(str::trim).filter(|s| !s.is_empty()).collect(),
        None => default_fields.to_vec(),
    };

    let mut out = String::new();
    for (i, field) in selected.iter().enumerate() {
        match *field {
            "contexts" => {
                render_contexts(&snapshot.contexts, color, raw, &mut out);
            }
            "traits" => {
                render_nested_traits(&snapshot.traits, color, raw, &mut out);
            }
            "meta" => {
                render_meta(&snapshot.meta, color, raw, &mut out);
            }
            _ => {
                return Err(format!("unknown field: {}", field));
            }
        }

        if i + 1 < selected.len() {
            out.push('\n');
        }
    }
    Ok(out)
}
```

#### 4.2.2: Implement Nested Traits Rendering

```rust
fn render_nested_traits(traits: &Value, color: bool, raw: bool, out: &mut String) {
    if let Value::Object(map) = traits {
        let heading = if color {
            "Traits:".bold().cyan().to_string()
        } else {
            "Traits:".to_string()
        };
        out.push_str(&heading);

        for (context, context_traits) in map {
            out.push('\n');
            out.push_str("  ");
            out.push_str(context);
            out.push_str(":");

            if let Value::Object(fields) = context_traits {
                for (field, value) in fields {
                    out.push('\n');
                    out.push_str("    ");
                    out.push_str(field);
                    out.push_str(" = ");
                    out.push_str(&colorize_value(&value.to_string(), color));
                }
            }
        }
    }
}
```

**Expected Output Format**:

```
Contexts: agent, ide

Traits:
  agent:
    id = cursor
  ide:
    id = vscode
  terminal:
    interactive = true
    color_level = full
    stdin = {"tty": true, "piped": false}
    stdout = {"tty": true, "piped": false}
    stderr = {"tty": true, "piped": false}
    supports_hyperlinks = true
```

### Task 4.3: Enhanced Check Command Output

**Files**: `src/main.rs`, `src/check.rs`

**Objective**: Implement enhanced output formatting that handles different
result types based on the new CLI behavior requirements.

#### 4.3.1: Define CheckResult Types

```rust
#[derive(Debug, Clone)]
pub enum CheckResult {
    Boolean(bool),
    String(String),
    Comparison { actual: String, expected: String, matched: bool },
}

impl CheckResult {
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
            CheckResult::Comparison { actual, expected, matched } => {
                if explain {
                    format!("{}  # {} == {}", matched, actual, expected)
                } else {
                    matched.to_string()
                }
            }
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            CheckResult::Boolean(b) => *b,
            CheckResult::String(s) => !s.is_empty(),
            CheckResult::Comparison { matched, .. } => *matched,
        }
    }
}
```

#### 4.3.2: Update JsonCheck Structure

```rust
#[derive(Debug, Serialize, Clone)]
struct JsonCheck {
    predicate: String,
    result: serde_json::Value,  // Changed to Value to support different types
    reason: Option<String>,
    signals: Option<BTreeMap<String, String>>,
}
```

#### 4.3.3: Update Output Results Function

```rust
fn output_results(results: &[JsonCheck], overall: bool, mode_any: bool, json: bool, explain: bool) {
    if json {
        let out = JsonOutput {
            overall,
            mode: if mode_any { "any" } else { "all" },
            checks: results,
        };

        if explain {
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        } else {
            println!("{}", serde_json::to_string(&out).unwrap());
        }
    } else {
        // Human-readable output
        if results.len() == 1 {
            let r = &results[0];
            let formatted_result = format_result_value(&r.result);
            if let Some(reason) = r.reason.as_ref().filter(|_| explain) {
                println!("{}  # reason: {}", formatted_result, reason);
            } else {
                println!("{}", formatted_result);
            }
        } else {
            println!("overall={}", overall);
            for r in results {
                let formatted_result = format_result_value(&r.result);
                if let Some(reason) = r.reason.as_ref().filter(|_| explain) {
                    println!("{}={}  # reason: {}", r.predicate, formatted_result, reason);
                } else {
                    println!("{}={}", r.predicate, formatted_result);
                }
            }
        }
    }
}

fn format_result_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => value.to_string(),
    }
}
```

### Task 4.4: CLI Behavior Implementation

**Files**: `src/check.rs`

**Objective**: Implement the specific CLI behavior outlined in the README
examples.

#### 4.4.1: Context vs Field Output Logic

The CLI behavior differs based on check type:

- **Context checks** (`agent`, `ide`, `ci`): Return boolean indicating presence
- **Field value checks** (`agent.id`, `terminal.interactive`): Return actual
  value
- **Field comparisons** (`agent.id=cursor`): Return boolean match result

#### 4.4.2: Update Evaluation Function

```rust
fn evaluate_nested_field(
    env: &EnvSense,
    path: &[String],
    expected_value: Option<&str>,
    registry: &FieldRegistry,
) -> (CheckResult, Option<String>, Option<BTreeMap<String, String>>) {
    let field_info = registry.resolve_field(path);
    let field_info = match field_info {
        Some(info) => info,
        None => return (
            CheckResult::Boolean(false),
            Some("unknown field".to_string()),
            None
        ),
    };

    // Navigate to the field value
    let actual_value = navigate_to_field(&env.traits, &field_info.path);

    let result = match expected_value {
        Some(expected) => {
            // Comparison mode - return boolean match result
            let matched = match field_info.field_type {
                FieldType::Boolean => {
                    let expected_bool = expected == "true";
                    actual_value.as_bool().unwrap_or(false) == expected_bool
                }
                FieldType::String | FieldType::OptionalString => {
                    actual_value.as_str() == Some(expected)
                }
                FieldType::ColorLevel => {
                    // Handle color level comparison
                    actual_value.as_str() == Some(expected)
                }
            };
            CheckResult::Comparison {
                actual: actual_value.as_str().unwrap_or("null").to_string(),
                expected: expected.to_string(),
                matched,
            }
        }
        None => {
            // Value display mode - return actual value
            match field_info.field_type {
                FieldType::Boolean => {
                    CheckResult::Boolean(actual_value.as_bool().unwrap_or(false))
                }
                FieldType::String | FieldType::OptionalString => {
                    CheckResult::String(
                        actual_value.as_str().unwrap_or("").to_string()
                    )
                }
                FieldType::ColorLevel => {
                    CheckResult::String(
                        actual_value.as_str().unwrap_or("none").to_string()
                    )
                }
            }
        }
    };

    (
        result,
        Some(format!("field: {}", path.join("."))),
        None
    )
}
```

#### 4.4.3: Context Evaluation

```rust
fn evaluate_context(env: &EnvSense, context: &str) -> (CheckResult, Option<String>, Option<BTreeMap<String, String>>) {
    let present = env.contexts.contains(&context.to_string());
    (
        CheckResult::Boolean(present),
        Some(format!("context: {}", context)),
        None
    )
}
```

### Task 4.5: Update Help Text Generation

**Files**: `src/main.rs`

**Objective**: Generate help text that shows the new field structure and
available predicates.

```rust
fn check_predicate_long_help() -> &'static str {
    static HELP: OnceLock<String> = OnceLock::new();
    HELP.get_or_init(|| {
        let registry = FieldRegistry::new();
        generate_help_text(&registry)
    })
    .as_str()
}

fn generate_help_text(registry: &FieldRegistry) -> String {
    let mut help = String::from("Available predicates:\n\n");

    help.push_str("Contexts (return boolean for presence):\n");
    for context in &["agent", "ide", "ci", "terminal"] {
        help.push_str(&format!("  {}\n", context));
    }

    help.push_str("\nFields (return actual value, or boolean if compared with =value):\n");
    let mut fields: Vec<_> = registry.fields.keys().collect();
    fields.sort();
    for field in fields {
        let info = &registry.fields[field];
        help.push_str(&format!("  {}  # {}\n", field, info.description));
    }

    help.push_str("\nExamples:\n");
    help.push_str("  envsense check agent              # true/false (context presence)\n");
    help.push_str("  envsense check agent.id           # \"cursor\" (field value)\n");
    help.push_str("  envsense check agent.id=cursor    # true/false (comparison)\n");
    help.push_str("  envsense check terminal.interactive # true/false (boolean field)\n");

    help
}
```

### Task 4.6: Update List Command

**Files**: `src/main.rs`

**Objective**: Update the list command to show available contexts and fields
separately.

```rust
fn list_checks() {
    let registry = FieldRegistry::new();

    println!("contexts:");
    for context in &["agent", "ide", "ci", "terminal"] {
        println!("  {}", context);
    }

    println!("fields:");
    let mut fields: Vec<_> = registry.fields.keys().collect();
    fields.sort();
    for field in fields {
        println!("  {}", field);
    }
}
```

### Task 4.7: Update CLI Argument Handling

**Files**: `src/main.rs`

**Objective**: Ensure CLI argument parsing works with the new predicate syntax.

#### 4.7.1: Verify CheckCmd Structure

```rust
#[derive(Args, Clone)]
pub struct CheckCmd {
    /// Predicates to evaluate
    #[arg(value_name = "PREDICATE")]
    pub predicates: Vec<String>,

    /// Show explanations for results
    #[arg(short, long)]
    pub explain: bool,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,

    /// Use ANY mode (default is ALL)
    #[arg(long)]
    pub any: bool,

    /// List available predicates
    #[arg(long)]
    pub list: bool,
}
```

#### 4.7.2: Update Check Command Handler

```rust
fn run_check(args: CheckCmd) -> Result<(), i32> {
    if args.list {
        list_checks();
        return Ok(());
    }

    if args.predicates.is_empty() {
        eprintln!("Error: no predicates specified");
        return Err(1);
    }

    let env = EnvSense::detect();
    let registry = FieldRegistry::new();
    let mut results = Vec::new();

    for predicate in &args.predicates {
        let parsed = match parse_with_warnings(predicate) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error parsing '{}': {:?}", predicate, e);
                return Err(1);
            }
        };

        let (result, reason, signals) = evaluate(&env, parsed, &registry);

        results.push(JsonCheck {
            predicate: predicate.clone(),
            result: match result {
                CheckResult::Boolean(b) => serde_json::Value::Bool(b),
                CheckResult::String(s) => serde_json::Value::String(s),
                CheckResult::Comparison { matched, .. } => serde_json::Value::Bool(matched),
            },
            reason,
            signals,
        });
    }

    let overall = if args.any {
        results.iter().any(|r| r.result.as_bool().unwrap_or(false))
    } else {
        results.iter().all(|r| r.result.as_bool().unwrap_or(false))
    };

    output_results(&results, overall, args.any, args.json, args.explain);

    if overall {
        Ok(())
    } else {
        Err(1)
    }
}
```

## Testing Requirements

### Unit Tests

**Files**: `tests/cli_output_formatting.rs`

#### 4.7.1: Context Check Tests

```rust
#[test]
fn test_context_checks_return_boolean() {
    // Test that context checks return boolean values
    let output = run_check_command(&["agent"]);
    assert!(output == "true" || output == "false");
}

#[test]
fn test_field_value_display() {
    // Test that field checks without comparison show actual values
    let output = run_check_command(&["agent.id"]);
    assert!(!output.is_empty());
    assert!(output != "true" && output != "false"); // Should be actual value
}

#[test]
fn test_field_comparison() {
    // Test that field comparisons return boolean
    let output = run_check_command(&["agent.id=cursor"]);
    assert!(output == "true" || output == "false");
}
```

#### 4.7.2: Output Formatting Tests

```rust
#[test]
fn test_json_output_structure() {
    let output = run_check_command_json(&["agent.id"]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(json["checks"].is_array());
    assert!(json["overall"].is_boolean());
    assert_eq!(json["mode"], "all");
}

#[test]
fn test_human_readable_output() {
    let output = run_info_command(&[]);
    assert!(output.contains("Contexts:"));
    assert!(output.contains("Traits:"));
}
```

### Integration Tests

**Files**: `tests/cli_integration.rs`

#### 4.7.3: End-to-End CLI Tests

```rust
#[test]
fn test_readme_examples_work() {
    // Verify all examples from README work as documented
    let examples = vec![
        ("agent", "boolean"),
        ("agent.id", "string"),
        ("agent.id=cursor", "boolean"),
        ("terminal.interactive", "boolean"),
    ];

    for (predicate, expected_type) in examples {
        let output = run_check_command(&[predicate]);
        match expected_type {
            "boolean" => assert!(output == "true" || output == "false"),
            "string" => assert!(!output.is_empty() && output != "true" && output != "false"),
            _ => panic!("Unknown expected type: {}", expected_type),
        }
    }
}
```

### Snapshot Tests

**Files**: `tests/info_snapshots.rs`

Update existing snapshot tests to work with new schema:

```rust
#[test]
fn test_info_output_snapshot() {
    let output = run_info_command(&[]);
    insta::assert_snapshot!(output);
}

#[test]
fn test_check_output_snapshots() {
    let predicates = vec!["agent", "agent.id", "terminal.interactive"];
    for predicate in predicates {
        let output = run_check_command(&[predicate]);
        insta::assert_snapshot!(format!("check_{}", predicate.replace(".", "_")), output);
    }
}
```

## Success Criteria

### Functional Requirements

- [ ] JSON output matches new schema structure with nested traits
- [ ] Human-readable output displays nested traits in clear hierarchy
- [ ] Context checks (`agent`, `ide`, `ci`) return boolean values
- [ ] Field value checks (`agent.id`) return actual string values
- [ ] Field comparisons (`agent.id=cursor`) return boolean match results
- [ ] Boolean fields (`terminal.interactive`) return boolean values
- [ ] Help text shows new field structure and examples
- [ ] List command shows available contexts and fields separately

### Technical Requirements

- [ ] All existing CLI tests pass
- [ ] New CLI behavior tests pass
- [ ] Snapshot tests updated and passing
- [ ] No breaking changes to JSON API structure
- [ ] Backward compatibility maintained where possible

### User Experience Requirements

- [ ] Output is intuitive and matches README examples
- [ ] Error messages are clear and helpful
- [ ] Help text is comprehensive and accurate
- [ ] Performance is not degraded

## Dependencies

This phase requires completion of:

- **Phase 1**: New schema structures (`NestedTraits`, `AgentTraits`, etc.)
- **Phase 2**: Parser and evaluation logic with `FieldRegistry` and
  `CheckResult`
- **Phase 3**: Detection system populating nested trait structures

## Risk Mitigation

### High-Risk Areas

1. **Output Format Changes**: Changes to CLI output may break existing scripts
2. **JSON Structure Changes**: API consumers may depend on current structure
3. **Behavior Changes**: New check behavior may confuse existing users

### Mitigation Strategies

1. **Extensive Testing**: Comprehensive test coverage for all output formats
2. **Backward Compatibility**: Maintain JSON structure compatibility where
   possible
3. **Clear Documentation**: Update all documentation with new examples
4. **Gradual Migration**: Provide migration tools and clear upgrade paths

## Implementation Order

1. **Task 4.1**: Update JSON output structure (lowest risk)
2. **Task 4.7**: Update CLI argument handling (foundation)
3. **Task 4.4**: Implement CLI behavior logic (core functionality)
4. **Task 4.3**: Enhanced check command output (builds on behavior)
5. **Task 4.2**: Update human-readable output rendering (visual)
6. **Task 4.5**: Update help text generation (documentation)
7. **Task 4.6**: Update list command (final feature)

## Validation

Before marking Phase 4 complete:

1. **Manual Testing**: Test all README examples manually
2. **Regression Testing**: Ensure no existing functionality is broken
3. **Performance Testing**: Verify no significant performance degradation
4. **Documentation Review**: Ensure all examples and documentation are accurate
5. **User Acceptance**: Get feedback on new CLI behavior from early users

## Next Steps

After Phase 4 completion, proceed to:

- **Phase 5**: Migration & Cleanup - Remove legacy code and provide migration
  tools

This phase establishes the final user-facing interface for the streamlined CLI,
making it critical to get the user experience right before moving to cleanup
phases.
