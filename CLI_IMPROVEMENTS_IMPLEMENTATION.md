# CLI Improvements Implementation Plan

## Overview

This document outlines the implementation plan for enhancing the envsense CLI
with improved error handling, output formatting, and user experience features as
detailed in `docs/planning/additional-cli-improvements.md`.

## Current Architecture Analysis

### Key Components

- **`src/main.rs`**: CLI entry point with `clap` argument parsing
- **`src/check.rs`**: Check command implementation with field registry and
  evaluation
- **CLI Structure**: Uses `clap` with subcommands (`info`, `check`)
- **Error Handling**: Basic error messages with exit codes
- **Output Formatting**: JSON and human-readable formats

### Current Limitations

1. Basic error messages without usage guidance
2. No predicate syntax validation
3. No flag combination validation
4. Limited output formatting options
5. Basic context listing without descriptions

## Implementation Plan

### Phase 1: Enhanced Error Handling (High Priority)

#### 1.1 Improved `check` Command Usage Errors

**Files to Modify:**

- `src/main.rs`: Update `run_check()` function
- `src/check.rs`: Add helper functions for usage display

**Implementation:**

```rust
// In src/main.rs - Update run_check function
fn run_check(args: CheckCmd) -> Result<(), i32> {
    // Validate flag combinations first
    if let Err(validation_error) = validate_check_flags(&args) {
        eprintln!("{}", validation_error);
        return Err(1);
    }

    if args.list {
        list_checks();
        return Ok(());
    }

    if args.predicates.is_empty() {
        display_check_usage_error();
        return Err(1);
    }

    // ... rest of function
}

fn display_check_usage_error() {
    eprintln!("Error: no predicates specified");
    eprintln!();
    eprintln!("Usage: envsense check <predicate> [<predicate>...]");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  envsense check agent                    # Check if running in an agent");
    eprintln!("  envsense check ide.cursor              # Check if Cursor IDE is active");
    eprintln!("  envsense check ci.github               # Check if in GitHub CI");
    eprintln!("  envsense check agent.id=cursor         # Check specific agent ID");
    eprintln!("  envsense check --list                  # List all available predicates");
    eprintln!();
    eprintln!("For more information, see: envsense check --help");
}
```

#### 1.2 Predicate Syntax Validation

**Files to Modify:**

- `src/check.rs`: Add validation functions
- `src/check.rs`: Update `ParseError` enum

**Implementation:**

```rust
// In src/check.rs - Add new error types
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

// Add validation function
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

// Add strict field validation
pub fn validate_field_path(path: &[String], registry: &FieldRegistry) -> Result<(), ParseError> {
    let field_path = path.join(".");

    if !registry.has_field(&field_path) {
        let context = &path[0];
        if registry.has_context(context) {
            let available_fields = registry.get_context_fields(context);
            let field_names: Vec<String> = available_fields
                .iter()
                .map(|(name, _)| name.clone())
                .collect();
            return Err(ParseError::InvalidFieldForContext(
                field_path,
                context.clone(),
                field_names.join(", ")
            ));
        } else {
            return Err(ParseError::FieldNotFound(field_path));
        }
    }

    Ok(())
}
```

#### 1.3 Flag Combination Validation

**Files to Modify:**

- `src/main.rs`: Add validation function
- `src/main.rs`: Update `CheckCmd` struct with validation

**Implementation:**

```rust
// In src/main.rs - Add flag validation
#[derive(Debug)]
enum FlagValidationError {
    ListWithEvaluationFlags,
    ListWithPredicates,
    ListWithQuiet,
}

impl std::fmt::Display for FlagValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlagValidationError::ListWithEvaluationFlags => {
                writeln!(f, "Error: invalid flag combination: --list cannot be used with --any or --all")?;
                writeln!(f)?;
                writeln!(f, "The --list flag shows available predicates, while --any/--all control evaluation logic.")?;
                writeln!(f, "These flags serve different purposes and cannot be combined.")?;
                writeln!(f)?;
                writeln!(f, "Usage examples:")?;
                writeln!(f, "  envsense check --list                    # List available predicates")?;
                writeln!(f, "  envsense check --any agent ide          # Check if ANY predicate is true")?;
                write!(f, "  envsense check --all agent ide          # Check if ALL predicates are true")
            }
            FlagValidationError::ListWithPredicates => {
                writeln!(f, "Error: invalid flag combination: --list cannot be used with predicates")?;
                writeln!(f)?;
                writeln!(f, "The --list flag shows all available predicates, so providing specific predicates is redundant.")?;
                writeln!(f)?;
                writeln!(f, "Usage examples:")?;
                writeln!(f, "  envsense check --list                    # List all available predicates")?;
                writeln!(f, "  envsense check agent                    # Check specific predicate")?;
                write!(f, "  envsense check agent ide                # Check multiple predicates")
            }
            FlagValidationError::ListWithQuiet => {
                writeln!(f, "Error: invalid flag combination: --list cannot be used with --quiet")?;
                writeln!(f)?;
                writeln!(f, "The --list flag is designed to show information, while --quiet suppresses output.")?;
                writeln!(f, "These flags have contradictory purposes and cannot be combined.")?;
                writeln!(f)?;
                writeln!(f, "Usage examples:")?;
                writeln!(f, "  envsense check --list                    # Show available predicates")?;
                write!(f, "  envsense check agent --quiet            # Check predicate quietly")
            }
        }
    }
}

fn validate_check_flags(args: &CheckCmd) -> Result<(), FlagValidationError> {
    if args.list {
        if args.any || args.all {
            return Err(FlagValidationError::ListWithEvaluationFlags);
        }
        if !args.predicates.is_empty() {
            return Err(FlagValidationError::ListWithPredicates);
        }
        if args.quiet {
            return Err(FlagValidationError::ListWithQuiet);
        }
    }
    Ok(())
}
```

### Phase 2: Enhanced Output Formatting (Medium Priority)

#### 2.1 Improved Context Listing

**Files to Modify:**

- `src/main.rs`: Update `list_checks()` function
- `src/check.rs`: Add context descriptions

**Implementation:**

```rust
// In src/check.rs - Add context descriptions
impl FieldRegistry {
    pub fn get_context_description(&self, context: &str) -> &str {
        match context {
            "agent" => "Agent environment detection",
            "ide" => "Integrated development environment",
            "ci" => "Continuous integration environment",
            "runtime" => "Runtime environment details",
            "os" => "Operating system information",
            "terminal" => "Terminal characteristics",
            "shell" => "Shell environment details",
            _ => "Context information"
        }
    }
}

// In src/main.rs - Update list_checks function
fn list_checks() {
    let registry = FieldRegistry::new();

    println!("Available contexts:");
    for context in registry.get_contexts() {
        println!("- {}: {}", context, registry.get_context_description(context));
    }

    println!("\nAvailable fields:");
    for context in registry.get_contexts() {
        let context_fields = registry.get_context_fields(context);
        if !context_fields.is_empty() {
            println!("\n  {} fields:", context);
            let mut sorted_fields = context_fields;
            sorted_fields.sort_by(|a, b| a.0.cmp(b.0));

            for (field_path, field_info) in sorted_fields {
                println!("    {:<25} # {}", field_path, field_info.description);
            }
        }
    }
}
```

#### 2.2 Hierarchical Info Display

**Files to Modify:**

- `src/main.rs`: Update `render_human()` function

**Implementation:**

```rust
// In src/main.rs - Add hierarchical display function
fn render_nested_value(value: &serde_json::Value, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);

    match value {
        serde_json::Value::Object(map) => {
            let mut result = String::new();
            for (key, val) in map {
                result.push_str(&format!("{}{}:\n", indent_str, key));
                match val {
                    serde_json::Value::Object(_) => {
                        result.push_str(&render_nested_value(val, indent + 1));
                    }
                    _ => {
                        result.push_str(&format!("{}  {}\n", indent_str, format_simple_value(val)));
                    }
                }
            }
            result
        }
        _ => format!("{}{}\n", indent_str, format_simple_value(value))
    }
}

fn format_simple_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => value.to_string()
    }
}
```

#### 2.3 Rainbow Color Level Display ✅ COMPLETED

**Actual Implementation:**

Rainbow colors were implemented but integrated with standard color controls
instead of using separate flags. The implementation uses the existing `colored`
crate rather than adding `owo-colors` to minimize dependencies.

**Key Decisions:**

- No separate `--rainbow` flag - rainbow colors activate automatically when
  colors are enabled
- Uses standard color controls (`NO_COLOR`, `--no-color`) for consistency
- Applied only to "truecolor" values for visual emphasis
- Standardized on single `colored` crate to avoid dependency conflicts

## Phase 2 Implementation Summary ✅ COMPLETED

### Final Output Format

The completed implementation creates a clean, YAML-compatible hierarchical
format:

```yaml
Contexts:
  - agent
  - ide

Traits:
  agent:
    id: cursor
  ci: none
  ide:
    id: cursor
  terminal:
    color_level: truecolor # Rainbow colored when colors enabled
    interactive: true
    stderr:
      piped: false
      tty: true
    stdin:
      piped: false
      tty: true
    stdout:
      piped: false
      tty: true
    supports_hyperlinks: true
```

### Key Implementation Changes

1. **Contexts Display**: Changed from inline `agent, ide` to bullet list format
2. **Hierarchical Display**: Made default for all info output (removed `--tree`
   flag)
3. **Format Syntax**: Changed from `key = value` to `key: value` for YAML
   compatibility
4. **Empty Traits**: Display as `context: none` with red coloring (same as false
   values)
5. **Indentation**: Consistent 2-space increments, contexts and traits aligned
   at same level
6. **Rainbow Colors**: Integrated with standard color controls, no separate
   flags needed
7. **Dependencies**: Standardized on `colored` crate, removed `owo-colors`
   dependency

### Files Modified

- **`src/main.rs`**:
  - Updated `list_checks()` with context descriptions and field formatting
  - Added `render_nested_value_with_rainbow()` for hierarchical display
  - Integrated rainbow color formatting with existing color system
  - Updated `InfoArgs` to remove `--tree` flag
  - Made hierarchical display the default behavior

- **`src/check.rs`**:
  - Added `get_context_description()` method to `FieldRegistry`
  - Updated predicate validation to allow hyphens in syntax

- **`Cargo.toml`**:
  - ~~Added `owo-colors` dependency~~ (later removed for consistency)
  - Standardized on existing `colored` crate

- **Test Files**:
  - `tests/cli_output_formatting.rs`: Comprehensive test suite for new
    formatting
  - `tests/cli.rs`: Updated existing tests for new output format
  - `tests/info_snapshots.rs`: Added snapshot test for new `--list` format

### Testing Coverage

- 12 new output formatting tests covering all new features
- Updated existing CLI tests for new format expectations
- Snapshot tests for format consistency
- Error handling tests for edge cases
- All 400+ existing tests continue to pass

### Phase 3: Configuration and Polish (Lower Priority)

#### 3.1 Configuration System

**Files to Create:**

- `src/config.rs`: Configuration management

**Implementation:**

```rust
// In src/config.rs - Configuration structure
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub error_handling: ErrorHandlingConfig,
    pub output_formatting: OutputFormattingConfig,
    pub validation: ValidationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig {
    pub strict_mode: bool,
    pub show_usage_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormattingConfig {
    pub context_descriptions: bool,
    pub nested_display: bool,
    pub rainbow_colors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub validate_predicates: bool,
    pub allowed_characters: String,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            error_handling: ErrorHandlingConfig {
                strict_mode: true,
                show_usage_on_error: true,
            },
            output_formatting: OutputFormattingConfig {
                context_descriptions: true,
                nested_display: true,
                rainbow_colors: true,
            },
            validation: ValidationConfig {
                validate_predicates: true,
                allowed_characters: "a-zA-Z0-9_.=".to_string(),
            },
        }
    }
}

impl CliConfig {
    pub fn load() -> Self {
        // Try to load from config file, fallback to default
        if let Some(config_path) = Self::config_file_path() {
            if let Ok(content) = std::fs::read_to_string(config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    fn config_file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("envsense");
            path.push("config.toml");
            path
        })
    }
}
```

#### 3.2 Additional CLI Flags

**Files to Modify:**

- `src/main.rs`: Add new flags

**Implementation:**

```rust
// In src/main.rs - Add new flags to CheckCmd
#[derive(Args, Clone)]
pub struct CheckCmd {
    // ... existing fields

    /// Use lenient mode (don't error on invalid fields)
    #[arg(long)]
    pub lenient: bool,

    /// Disable rainbow colors
    #[arg(long)]
    pub no_rainbow: bool,

    /// Show context descriptions in list mode
    #[arg(long, requires = "list")]
    pub descriptions: bool,
}

// In src/main.rs - Add new flags to InfoArgs
#[derive(Args, Clone)]
struct InfoArgs {
    // ... existing fields

    /// Use tree structure for nested display
    #[arg(long)]
    tree: bool,

    /// Compact output without descriptions
    #[arg(long)]
    compact: bool,
}
```

## Testing Strategy

### 3.1 Error Handling Tests

**Files to Create:**

- `tests/cli_error_handling.rs`

**Test Cases:**

- No predicates provided
- Invalid predicate syntax
- Invalid flag combinations
- Invalid field paths
- Special character validation

### 3.2 Output Formatting Tests

**Files to Create:**

- `tests/cli_output_formatting.rs`

**Test Cases:**

- Context listing format
- Hierarchical display
- Rainbow color output
- JSON vs human formatting

### 3.3 Integration Tests

**Files to Update:**

- `tests/cli.rs`: Add comprehensive CLI behavior tests

**Test Scenarios:**

- End-to-end error scenarios
- Flag combination validation
- Output consistency
- Performance impact measurement

## Migration Strategy

### Backward Compatibility

1. **Existing Commands**: All current commands continue to work unchanged
2. **New Flags**: All new flags are optional with sensible defaults
3. **Error Messages**: Enhanced but maintain similar structure
4. **Configuration**: Optional configuration file, defaults preserve current
   behavior

### Rollout Plan

1. **Phase 1 (High Priority)**:
   - Enhanced error messages
   - Basic syntax validation
   - Flag combination validation

2. **Phase 2 (Medium Priority)**:
   - Improved output formatting
   - Context descriptions
   - Hierarchical display

3. **Phase 3 (Lower Priority)**:
   - Configuration system
   - Rainbow colors
   - Advanced features

### Performance Considerations

- **Validation Overhead**: Minimal regex validation on predicates
- **Color Detection**: Cache terminal capability detection
- **Field Registry**: Use existing efficient registry system
- **Memory Usage**: Lazy loading of descriptions and help text

## Implementation Checklist

### Phase 1: Error Handling ✅ COMPLETED

- [x] Update `run_check()` with improved error messages
- [x] Add `validate_check_flags()` function
- [x] Add `display_check_usage_error()` function
- [x] Extend `ParseError` enum with new error types
- [x] Add `validate_predicate_syntax()` function
- [x] Add `validate_field_path()` function with strict mode
- [x] Add tests for all error scenarios

### Phase 2: Output Formatting ✅ COMPLETED

- [x] Add context descriptions to `FieldRegistry`
- [x] Update `list_checks()` with improved formatting
- [x] Add `render_nested_value_with_rainbow()` function
- [x] Implement hierarchical display for info command (made default)
- [x] Add rainbow color formatting capability
- [x] ~~Add `--rainbow` and `--no-rainbow` flags~~ (integrated with standard
      color controls)
- [x] Update snapshot tests for new formats
- [x] Remove separate `--tree` flag (hierarchical display is now default)
- [x] Change format from `key = value` to `key: value` (YAML-compatible)
- [x] Align contexts and traits indentation for visual consistency
- [x] Handle empty traits as `context: none` with red coloring
- [x] Comprehensive test coverage for all new formatting features

### Phase 3: Configuration

- [ ] Create `src/config.rs` with configuration structures
- [ ] Add TOML configuration file support
- [ ] Add additional CLI flags (`--lenient`, `--descriptions`, `--tree`)
- [ ] Add configuration loading and validation
- [ ] Add documentation for configuration options
- [ ] Add tests for configuration system

### Documentation Updates

- [ ] Update `README.md` with new CLI features
- [ ] Add configuration documentation
- [ ] Update help text and examples
- [ ] Add migration guide for power users

### CI/CD Updates

- [ ] Update GitHub Actions for new dependencies
- [ ] Add performance regression tests
- [ ] Update snapshot tests
- [ ] Add integration tests for new features

## Dependencies

### New Crate Dependencies

```toml
[dependencies]
# For rainbow colors
owo-colors = "4.0"

# For configuration files  
toml = "0.8"
dirs = "5.0"

# For regex validation
regex = "1.0"
```

### Development Dependencies

```toml
[dev-dependencies]
# For testing configuration
tempfile = "3.0"
```

## Risk Assessment

### Low Risk

- Enhanced error messages (additive)
- New optional flags (backward compatible)
- Configuration system (optional)

### Medium Risk

- Flag validation (could break invalid usage)
- Strict field validation (behavior change)
- Output format changes (could affect scripts)

### Mitigation Strategies

- Feature flags for breaking changes
- Comprehensive testing
- Clear migration documentation
- Gradual rollout plan
