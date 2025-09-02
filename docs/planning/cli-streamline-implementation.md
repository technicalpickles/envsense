# CLI Streamlining Implementation Plan

## Overview

This document outlines a phased approach to implementing the CLI streamlining
changes outlined in `streamlining-cli.md`. The implementation is designed to
minimize risk while providing clear migration paths for users.

## Implementation Strategy

### Core Principles

1. **Backward Compatibility**: Maintain support for existing syntax during
   transition
2. **Incremental Delivery**: Each phase delivers working functionality
3. **Clear Migration Path**: Provide tools and documentation for users
4. **Risk Mitigation**: Extensive testing at each phase

### High-Level Timeline

- **Phase 1**: Foundation & Schema (2-3 weeks)
- **Phase 2**: Parser & Evaluation (2-3 weeks)
- **Phase 3**: Detection System (2-3 weeks)
- **Phase 4**: CLI Integration (1-2 weeks)
- **Phase 5**: Migration & Cleanup (1-2 weeks)

**Total Estimated Time**: 8-13 weeks

## Phase 1: Foundation & Schema

### Objective

Establish the new nested schema structure while maintaining backward
compatibility.

### Tasks

#### 1.1 Create New Trait Structures

**Files**: `src/schema.rs`, `src/traits/`

```rust
// New nested trait structures
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct AgentTraits {
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct IdeTraits {
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct TerminalTraits {
    pub interactive: bool,
    pub color_level: ColorLevel,
    pub stdin: StreamInfo,
    pub stdout: StreamInfo,
    pub stderr: StreamInfo,
    pub supports_hyperlinks: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct StreamInfo {
    pub tty: bool,
    pub piped: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct CiTraits {
    pub id: Option<String>,
    pub vendor: Option<String>,
    pub name: Option<String>,
    pub is_pr: Option<bool>,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct NestedTraits {
    pub agent: AgentTraits,
    pub ide: IdeTraits,
    pub terminal: TerminalTraits,
    pub ci: CiTraits,
}
```

#### 1.2 Update Main Schema

**Files**: `src/schema.rs`

```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, DetectionMergerDerive)]
pub struct EnvSense {
    pub contexts: Vec<String>,  // Simplified from Contexts struct
    pub traits: NestedTraits,   // New nested structure
    pub evidence: Vec<Evidence>,
    pub version: String,
}

// Maintain backward compatibility
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct LegacyEnvSense {
    pub contexts: Contexts,
    pub facets: Facets,
    pub traits: Traits,
    pub evidence: Vec<Evidence>,
    pub version: String,
}
```

#### 1.3 Schema Version Management

**Files**: `src/schema.rs`

```rust
pub const SCHEMA_VERSION: &str = "0.3.0";
pub const LEGACY_SCHEMA_VERSION: &str = "0.2.0";

impl EnvSense {
    pub fn to_legacy(&self) -> LegacyEnvSense {
        // Convert new schema to legacy format
    }

    pub fn from_legacy(legacy: &LegacyEnvSense) -> Self {
        // Convert legacy schema to new format
    }
}
```

#### 1.4 Update Macro System

**Files**: `envsense-macros/src/`, `envsense-macros-impl/src/`

- Extend `DetectionMergerDerive` to handle nested structures
- Add support for nested object merging
- Maintain backward compatibility for flat structures

#### 1.5 Tests & Validation

**Files**: `tests/`, `src/schema.rs`

```rust
#[test]
fn new_schema_serialization() {
    let env = EnvSense::default();
    let json = serde_json::to_string(&env).unwrap();
    assert!(json.contains("\"version\":\"0.3.0\""));
    assert!(json.contains("\"traits\":"));
}

#[test]
fn legacy_conversion() {
    let legacy = LegacyEnvSense::default();
    let new = EnvSense::from_legacy(&legacy);
    let back = new.to_legacy();
    assert_eq!(legacy, back);
}
```

### Success Criteria

- [ ] New schema structures compile and serialize correctly
- [ ] Legacy conversion functions work bidirectionally
- [ ] All existing tests pass with new schema
- [ ] Macro system supports nested structures
- [ ] Schema version bumped to 0.3.0

### Dependencies

- None (foundation phase)

### Risk Mitigation

- Extensive testing of schema conversions
- Maintain both schemas during transition
- Clear documentation of changes

---

## Phase 2: Parser & Evaluation

### Objective

Implement new predicate syntax parser with dot notation support.

### Tasks

#### 2.1 New Parser Implementation

**Files**: `src/check.rs`

```rust
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Check {
    Context(String),
    NestedField { path: Vec<String>, value: Option<String> },
    LegacyFacet { key: String, value: String },
    LegacyTrait { key: String },
}

pub fn parse(input: &str) -> Result<Check, ParseError> {
    // Support both new and legacy syntax
    if let Some(rest) = input.strip_prefix("facet:") {
        // Legacy facet parsing
        parse_legacy_facet(rest)
    } else if let Some(rest) = input.strip_prefix("trait:") {
        // Legacy trait parsing
        parse_legacy_trait(rest)
    } else if input.contains('.') {
        // New dot notation parsing
        parse_nested_field(input)
    } else {
        // Context parsing (unchanged)
        Ok(Check::Context(input.to_string()))
    }
}

fn parse_nested_field(input: &str) -> Result<Check, ParseError> {
    // Parse "context.field[.subfield]=value" or "context.field"
    let (path, value) = if let Some((path_str, value_str)) = input.split_once('=') {
        (path_str, Some(value_str.to_string()))
    } else {
        (input, None)
    };

    let path_parts: Vec<String> = path.split('.').map(|s| s.to_string()).collect();
    if path_parts.len() < 2 {
        return Err(ParseError::Invalid);
    }

    Ok(Check::NestedField { path: path_parts, value })
}
```

#### 2.2 Field Registry System

**Files**: `src/check.rs`

```rust
#[derive(Debug, Clone)]
pub struct FieldRegistry {
    fields: HashMap<String, FieldInfo>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub field_type: FieldType,
    pub path: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    Boolean,
    String,
    OptionalString,
    ColorLevel,
}

impl FieldRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            fields: HashMap::new(),
        };
        registry.register_fields();
        registry
    }

    fn register_fields(&mut self) {
        // Register all available fields with their types and paths
        self.register("agent.id", FieldType::OptionalString, vec!["agent", "id"]);
        self.register("terminal.interactive", FieldType::Boolean, vec!["terminal", "interactive"]);
        self.register("terminal.stdin.tty", FieldType::Boolean, vec!["terminal", "stdin", "tty"]);
        // ... more fields
    }

    pub fn resolve_field(&self, path: &[String]) -> Option<&FieldInfo> {
        let key = path.join(".");
        self.fields.get(&key)
    }
}
```

#### 2.3 Updated Evaluation Logic

**Files**: `src/check.rs`, `src/main.rs`

```rust
fn evaluate(
    env: &EnvSense,
    parsed: ParsedCheck,
    registry: &FieldRegistry,
) -> (bool, Option<String>, Option<BTreeMap<String, String>>) {
    let (result, reason, signals) = match parsed.check {
        Check::Context(ctx) => {
            // Context evaluation (unchanged)
            evaluate_context(env, &ctx)
        }
        Check::NestedField { path, value } => {
            // New nested field evaluation
            evaluate_nested_field(env, &path, value.as_deref(), registry)
        }
        Check::LegacyFacet { key, value } => {
            // Legacy facet evaluation (deprecated)
            evaluate_legacy_facet(env, &key, &value)
        }
        Check::LegacyTrait { key } => {
            // Legacy trait evaluation (deprecated)
            evaluate_legacy_trait(env, &key)
        }
    };

    if parsed.negated {
        (!result, reason, signals)
    } else {
        (result, reason, signals)
    }
}

fn evaluate_nested_field(
    env: &EnvSense,
    path: &[String],
    expected_value: Option<&str>,
    registry: &FieldRegistry,
) -> (bool, Option<String>, Option<BTreeMap<String, String>>) {
    let field_info = registry.resolve_field(path);
    let field_info = match field_info {
        Some(info) => info,
        None => return (false, Some("unknown field".to_string()), None),
    };

    // Navigate to the field value
    let actual_value = navigate_to_field(&env.traits, &field_info.path);

    // Compare based on field type
    let result = match field_info.field_type {
        FieldType::Boolean => {
            let expected = expected_value.map(|v| v == "true").unwrap_or(true);
            actual_value.as_bool().unwrap_or(false) == expected
        }
        FieldType::String | FieldType::OptionalString => {
            match expected_value {
                Some(expected) => actual_value.as_str() == Some(expected),
                None => actual_value.is_some(),
            }
        }
        FieldType::ColorLevel => {
            // Handle color level comparison
            true // Implementation needed
        }
    };

    (result, Some(format!("field: {}", path.join("."))), None)
}
```

#### 2.4 Help Text Generation

**Files**: `src/main.rs`

```rust
fn generate_help_text(registry: &FieldRegistry) -> String {
    let mut help = String::from("Available predicates:\n\n");

    help.push_str("Contexts:\n");
    for context in &["agent", "ide", "ci"] {
        help.push_str(&format!("  {}\n", context));
    }

    help.push_str("\nFields:\n");
    for (field, info) in &registry.fields {
        help.push_str(&format!("  {}\n", field));
    }

    help
}
```

#### 2.5 Enhanced Output Formatting

**Files**: `src/check.rs`

```rust
#[derive(Debug)]
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
}
```

#### 2.6 Deprecation Warnings

**Files**: `src/check.rs`

```rust
fn parse_with_warnings(input: &str) -> Result<Check, ParseError> {
    let result = parse(input)?;

    // Warn about legacy syntax
    if input.starts_with("facet:") || input.starts_with("trait:") {
        eprintln!("Warning: Legacy syntax '{}' is deprecated. Use dot notation instead.", input);
    }

    Ok(result)
}
```

### Success Criteria

- [ ] New dot notation parser handles all required syntax
- [ ] Field registry correctly maps all fields
- [ ] Evaluation logic works for nested structures
- [ ] Legacy syntax still works with deprecation warnings
- [ ] Help text shows new field structure
- [ ] All parser tests pass
- [ ] Enhanced output formatting handles different result types
- [ ] CLI output matches README examples (boolean for contexts, values for
      fields)

### CLI Behavior Changes

Based on the updated README examples, the new CLI behavior includes:

#### Context Checks (Boolean Results)

```bash
envsense check agent        # => true/false (boolean result)
envsense check ide          # => true/false (boolean result)
envsense check ci           # => true/false (boolean result)
```

#### Field Value Checks (String Results)

```bash
envsense check agent.id     # => "cursor" (shows actual value)
envsense check ide.id       # => "vscode" (shows actual value)
envsense check terminal.interactive  # => true/false (boolean result)
```

#### Key Implementation Requirements

1. **Context Evaluation**: `agent`, `ide`, `ci` return boolean based on presence
2. **Field Value Display**: `agent.id` without `=value` shows the actual value
3. **Boolean Fields**: `terminal.interactive` returns boolean result
4. **String Fields**: `agent.id` returns string value when no comparison
   specified

#### Updated Evaluation Logic

The evaluation logic needs to handle three cases:

- **Context checks**: Return boolean for context presence
- **Field existence checks**: Return boolean for field existence (when no value
  specified)
- **Field value comparisons**: Return boolean for value matches (when `=value`
  specified)

#### CLI Output Behavior Examples

Based on the README examples, the CLI should behave as follows:

```bash
# Context checks return boolean
envsense check agent        # => true (if agent detected)
envsense check ide          # => false (if no IDE detected)

# Field checks without comparison return actual values
envsense check agent.id     # => "cursor" (shows actual agent ID)
envsense check ide.id       # => "vscode" (shows actual IDE ID)

# Field checks with comparison return boolean
envsense check agent.id=cursor    # => true (if agent.id == "cursor")
envsense check ide.id=vscode      # => true (if ide.id == "vscode")

# Boolean field checks return boolean values
envsense check terminal.interactive  # => true (if terminal is interactive)
```

#### Implementation Requirements

1. **Output Formatting**: Different output formats for different check types
2. **Value Extraction**: Ability to extract and display actual field values
3. **Comparison Logic**: Support for both existence checks and value comparisons
4. **Type Handling**: Proper handling of boolean vs string field types

### Dependencies

- Phase 1 (new schema structures)

### Risk Mitigation

- Extensive parser testing with edge cases
- Gradual deprecation with clear warnings
- Fallback to legacy evaluation if needed

---

## Phase 3: Detection System

### Objective

Update detectors to populate the new nested trait structure.

### Tasks

#### 3.1 Update Terminal Detector

**Files**: `src/detectors/terminal.rs`

```rust
impl Detector for TerminalDetector {
    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection {
            confidence: TERMINAL,
            ..Default::default()
        };

        // Create nested terminal traits
        let terminal_traits = TerminalTraits {
            interactive: snap.is_tty_stdin() && snap.is_tty_stdout(),
            color_level: detect_color_level(snap),
            stdin: StreamInfo {
                tty: snap.is_tty_stdin(),
                piped: !snap.is_tty_stdin(),
            },
            stdout: StreamInfo {
                tty: snap.is_tty_stdout(),
                piped: !snap.is_tty_stdout(),
            },
            stderr: StreamInfo {
                tty: snap.is_tty_stderr(),
                piped: !snap.is_tty_stderr(),
            },
            supports_hyperlinks: detect_hyperlinks(snap),
        };

        // Convert to JSON patches for nested structure
        detection.traits_patch.insert(
            "terminal".to_string(),
            serde_json::to_value(terminal_traits).unwrap(),
        );

        // Add evidence
        detection.evidence.push(
            Evidence::tty_trait("terminal.interactive", terminal_traits.interactive)
                .with_supports(vec!["terminal.interactive".into()])
        );

        detection
    }
}
```

#### 3.2 Update Agent Detector

**Files**: `src/detectors/agent_declarative.rs`

```rust
impl Detector for DeclarativeAgentDetector {
    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        // Detect agent using existing logic
        if let Some(agent_id) = detect_agent_id(snap) {
            let agent_traits = AgentTraits {
                id: Some(agent_id.clone()),
            };

            detection.traits_patch.insert(
                "agent".to_string(),
                serde_json::to_value(agent_traits).unwrap(),
            );

            detection.contexts_add.push("agent".to_string());

            // Add evidence
            detection.evidence.push(
                Evidence::env_var("AGENT_ID", agent_id)
                    .with_supports(vec!["agent.id".into()])
            );
        }

        detection
    }
}
```

#### 3.3 Update IDE Detector

**Files**: `src/detectors/ide_declarative.rs`

```rust
impl Detector for DeclarativeIdeDetector {
    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        // Detect IDE using existing logic
        if let Some(ide_id) = detect_ide_id(snap) {
            let ide_traits = IdeTraits {
                id: Some(ide_id.clone()),
            };

            detection.traits_patch.insert(
                "ide".to_string(),
                serde_json::to_value(ide_traits).unwrap(),
            );

            detection.contexts_add.push("ide".to_string());

            // Add evidence
            detection.evidence.push(
                Evidence::env_var("IDE_ID", ide_id)
                    .with_supports(vec!["ide.id".into()])
            );
        }

        detection
    }
}
```

#### 3.4 Update CI Detector

**Files**: `src/detectors/ci_declarative.rs`

```rust
impl Detector for DeclarativeCiDetector {
    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        // Detect CI using existing logic
        if let Some(ci_info) = detect_ci_info(snap) {
            let ci_traits = CiTraits {
                id: ci_info.id,
                vendor: ci_info.vendor,
                name: ci_info.name,
                is_pr: ci_info.is_pr,
                branch: ci_info.branch,
            };

            detection.traits_patch.insert(
                "ci".to_string(),
                serde_json::to_value(ci_traits).unwrap(),
            );

            detection.contexts_add.push("ci".to_string());

            // Add evidence
            detection.evidence.push(
                Evidence::env_var("CI", "true")
                    .with_supports(vec!["ci.id".into()])
            );
        }

        detection
    }
}
```

#### 3.5 Update Macro System for Nested Merging

**Files**: `envsense-macros-impl/src/lib.rs`

```rust
// Extend macro to handle nested object merging
fn generate_nested_merge_impl(input: &DeriveInput) -> TokenStream {
    // Generate merging logic for nested structures
    // Handle cases where traits_patch contains nested objects
    // Merge them into the appropriate nested trait fields
}
```

#### 3.6 Update Engine for Context Handling

**Files**: `src/engine.rs`

```rust
impl DetectionEngine {
    pub fn detect_from_snapshot(&self, snapshot: &EnvSnapshot) -> EnvSense {
        let mut result = EnvSense {
            contexts: Vec::new(),  // Now a Vec<String>
            traits: NestedTraits::default(),
            evidence: Vec::new(),
            version: SCHEMA_VERSION.to_string(),
        };

        // Collect detections
        let detections: Vec<envsense_macros::Detection> = self
            .detectors
            .iter()
            .map(|detector| {
                let detection = detector.detect(snapshot);
                envsense_macros::Detection {
                    contexts_add: detection.contexts_add,
                    traits_patch: detection.traits_patch,
                    facets_patch: detection.facets_patch, // Legacy support
                    evidence: detection.evidence.into_iter().map(|e| serde_json::to_value(e).unwrap()).collect(),
                    confidence: detection.confidence,
                }
            })
            .collect();

        // Merge detections
        result.merge_detections(&detections);
        result
    }
}
```

### Success Criteria

- [ ] All detectors populate nested trait structures
- [ ] Macro system correctly merges nested objects
- [ ] Evidence collection works with new field paths
- [ ] Contexts are properly collected as Vec<String>
- [ ] All detection tests pass
- [ ] JSON output matches new schema structure

### Dependencies

- Phase 1 (schema structures)
- Phase 2 (field registry)

### Risk Mitigation

- Extensive testing of detection logic
- Maintain evidence collection for debugging
- Verify JSON output structure

---

## Phase 4: CLI Integration

### Objective

Update CLI output rendering and user interface for new schema.

### Tasks

#### 4.1 Update JSON Output

**Files**: `src/main.rs`

```rust
fn collect_snapshot() -> Snapshot {
    let env = EnvSense::detect();

    Snapshot {
        contexts: env.contexts,  // Now Vec<String>
        traits: serde_json::to_value(env.traits).unwrap(),  // Nested structure
        facets: json!({}),  // Empty for new schema
        meta: json!({
            "schema_version": env.version,
        }),
        evidence: serde_json::to_value(env.evidence).unwrap(),
    }
}
```

#### 4.2 Update Human-Readable Output

**Files**: `src/main.rs`

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

#### 4.3 Update Check Command Output

**Files**: `src/main.rs`

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
            if let Some(reason) = r.reason.as_ref().filter(|_| explain) {
                println!("{}  # reason: {}", r.result, reason);
            } else {
                println!("{}", r.result);
            }
        } else {
            println!("overall={}", overall);
            for r in results {
                if let Some(reason) = r.reason.as_ref().filter(|_| explain) {
                    println!("{}={}  # reason: {}", r.predicate, r.result, reason);
                } else {
                    println!("{}={}", r.predicate, r.result);
                }
            }
        }
    }
}
```

#### 4.3.1 Enhanced Output Formatting

The new CLI behavior requires enhanced output formatting to handle different
types of results:

```rust
fn format_check_result(
    predicate: &str,
    result: &CheckResult,
    explain: bool,
) -> String {
    match result {
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

#[derive(Debug)]
enum CheckResult {
    Boolean(bool),
    String(String),
    Comparison { actual: String, expected: String, matched: bool },
}
```

#### 4.3.2 Context vs Field Output

The output format differs based on the check type:

- **Context checks** (`agent`, `ide`, `ci`): Return boolean
- **Field checks** (`agent.id`, `terminal.interactive`): Return actual value
- **Field comparisons** (`agent.id=cursor`): Return boolean match result

#### 4.4 Update Help Text

**Files**: `src/main.rs`

```rust
fn check_predicate_long_help() -> &'static str {
    static HELP: OnceLock<String> = OnceLock::new();
    HELP.get_or_init(|| {
        let registry = FieldRegistry::new();
        generate_help_text(&registry)
    })
    .as_str()
}
```

#### 4.5 Update List Command

**Files**: `src/main.rs`

```rust
fn list_checks() {
    let registry = FieldRegistry::new();

    println!("contexts:");
    for context in &["agent", "ide", "ci"] {
        println!("  {}", context);
    }

    println!("fields:");
    for (field, info) in &registry.fields {
        println!("  {}", field);
    }
}
```

### Success Criteria

- [ ] JSON output matches new schema structure
- [ ] Human-readable output shows nested traits clearly
- [ ] Check command works with new predicate syntax
- [ ] Help text shows new field structure
- [ ] List command shows available fields
- [ ] All CLI tests pass

### Dependencies

- Phase 1 (schema structures)
- Phase 2 (parser & evaluation)
- Phase 3 (detection system)

### Risk Mitigation

- Extensive CLI testing
- Verify output formats
- Test with real-world examples

---

## Phase 5: Migration & Cleanup

### Objective

Complete migration to new schema and remove legacy code.

### Tasks

#### 5.1 Migration Tools

**Files**: `src/migration.rs`

```rust
pub struct MigrationTools;

impl MigrationTools {
    /// Convert legacy predicate syntax to new syntax
    pub fn migrate_predicate(legacy: &str) -> Result<String, String> {
        match legacy {
            s if s.starts_with("facet:agent_id=") => {
                let value = s.strip_prefix("facet:agent_id=").unwrap();
                Ok(format!("agent.id={}", value))
            }
            s if s.starts_with("facet:ide_id=") => {
                let value = s.strip_prefix("facet:ide_id=").unwrap();
                Ok(format!("ide.id={}", value))
            }
            s if s.starts_with("trait:is_interactive") => {
                Ok("terminal.interactive".to_string())
            }
            // ... more mappings
            _ => Err("Unknown legacy predicate".to_string()),
        }
    }

    /// Convert legacy JSON schema to new schema
    pub fn migrate_json(legacy_json: &str) -> Result<String, String> {
        let legacy: LegacyEnvSense = serde_json::from_str(legacy_json)
            .map_err(|e| format!("Invalid legacy JSON: {}", e))?;

        let new = EnvSense::from_legacy(&legacy);
        serde_json::to_string_pretty(&new)
            .map_err(|e| format!("Failed to serialize: {}", e))
    }
}
```

#### 5.2 CLI Migration Commands

**Files**: `src/main.rs`

```rust
#[derive(Subcommand)]
enum Commands {
    /// Show what envsense knows
    Info(InfoArgs),
    /// Evaluate predicates against the environment
    Check(CheckCmd),
    /// Migration utilities
    Migrate(MigrateCmd),
}

#[derive(Args, Clone)]
struct MigrateCmd {
    /// Convert legacy predicate to new syntax
    #[arg(long, value_name = "PREDICATE")]
    predicate: Option<String>,

    /// Convert legacy JSON to new schema
    #[arg(long, value_name = "FILE")]
    json: Option<String>,

    /// Show migration guide
    #[arg(long)]
    guide: bool,
}

fn run_migrate(args: MigrateCmd) -> Result<(), i32> {
    if let Some(predicate) = args.predicate {
        match MigrationTools::migrate_predicate(&predicate) {
            Ok(new) => println!("{}", new),
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(1);
            }
        }
    } else if let Some(file) = args.json {
        let content = std::fs::read_to_string(&file)
            .map_err(|_| 1)?;
        match MigrationTools::migrate_json(&content) {
            Ok(new) => println!("{}", new),
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(1);
            }
        }
    } else if args.guide {
        print_migration_guide();
    } else {
        eprintln!("Use --help to see migration options");
        return Err(1);
    }
    Ok(())
}
```

#### 5.3 Remove Legacy Code

**Files**: Multiple

- Remove `Contexts`, `Facets`, `Traits` structs
- Remove legacy parser logic
- Remove legacy evaluation functions
- Remove backward compatibility layers

#### 5.4 Update Documentation

**Files**: `README.md`, `docs/`

- Update all examples to use new syntax
- Remove references to legacy features
- Update API documentation
- Create migration guide

#### 5.5 Final Testing

**Files**: `tests/`

```rust
#[test]
fn migration_completeness() {
    // Test that all legacy predicates can be migrated
    let legacy_predicates = vec![g
        "facet:agent_id=cursor",
        "facet:ide_id=vscode",
        "trait:is_interactive",
        // ... all legacy predicates
    ];

    for legacy in legacy_predicates {
        let migrated = MigrationTools::migrate_predicate(legacy).unwrap();
        // Verify migrated predicate works
        let env = EnvSense::detect();
        let registry = FieldRegistry::new();
        let parsed = parse(&migrated).unwrap();
        let (result, _, _) = evaluate(&env, parsed, &registry);
        // Should produce same result as legacy evaluation
    }
}
```

### Success Criteria

- [ ] Migration tools work correctly
- [ ] All legacy code removed
- [ ] Documentation updated
- [ ] All tests pass
- [ ] No deprecation warnings in codebase
- [ ] Migration guide available

### Dependencies

- All previous phases

### Risk Mitigation

- Comprehensive testing of migration tools
- Gradual removal of legacy code
- Clear communication to users

---

## Testing Strategy

### Unit Tests

- **Schema**: Serialization/deserialization, conversions
- **Parser**: All predicate syntax variations
- **Evaluation**: Field resolution, type checking
- **Detectors**: Nested trait population
- **CLI**: Output formatting, error handling

### CLI Behavior Tests

- **Context Checks**: Verify `agent`, `ide`, `ci` return boolean
- **Field Value Display**: Verify `agent.id` shows actual value
- **Field Comparisons**: Verify `agent.id=cursor` returns boolean match
- **Output Formatting**: Verify different result types format correctly
- **README Examples**: All examples in README work as documented

### Integration Tests

- **End-to-end**: Full detection pipeline
- **Migration**: Legacy to new schema conversion
- **Performance**: No regression in detection speed

### Snapshot Tests

- **JSON Output**: Verify schema structure
- **Human Output**: Verify formatting
- **CLI Commands**: Verify behavior

### Manual Testing

- **Real Environments**: Test in actual CI, IDE, agent environments
- **Migration**: Test migration tools with real user data
- **Documentation**: Verify all examples work

## Risk Management

### High-Risk Areas

1. **Schema Changes**: Breaking changes affect all consumers
2. **Parser Complexity**: New syntax may have edge cases
3. **Migration**: Users may lose functionality during transition

### Mitigation Strategies

1. **Gradual Migration**: Support both schemas during transition
2. **Extensive Testing**: Comprehensive test coverage
3. **Clear Communication**: Detailed migration guides and warnings
4. **Rollback Plan**: Ability to revert to legacy schema if needed

## Success Metrics

### Technical Metrics

- [ ] All tests pass
- [ ] No performance regression
- [ ] Schema version successfully bumped to 0.3.0
- [ ] Zero deprecation warnings

### User Experience Metrics

- [ ] Migration tools work for all use cases
- [ ] New syntax is more intuitive (user feedback)
- [ ] Documentation is clear and complete
- [ ] No breaking changes for critical use cases

### Community Metrics

- [ ] Successful migration of existing users
- [ ] Positive feedback on new interface
- [ ] No significant issues reported during transition

## Conclusion

This phased approach provides a clear path to implementing the CLI streamlining
changes while minimizing risk and ensuring a smooth migration for users. Each
phase builds on the previous one and delivers working functionality, allowing
for early feedback and course correction if needed.

The key to success is maintaining backward compatibility during the transition
period and providing clear migration tools and documentation for users. The end
result will be a much more intuitive and maintainable interface that better
serves the needs of the envsense community.
