# Extending envsense

This document describes how to add new **contexts**, **facets**, **traits**, and **detectors**.
The goal is to keep detection logic deterministic, evidence-backed, and schema-stable.

---

## General Principles

* **Schema is a contract**: Any new field requires a schema bump (`SCHEMA_VERSION`) and updated tests.
* **Evidence first**: Every `true` claim should be backed by at least one `Evidence` item.
* **Precedence rules** remain consistent:

  1. User overrides
  2. Explicit env vars
  3. Execution channel markers
  4. Process ancestry (opt-in)
  5. Heuristics

---

## Adding a New Context

1. Add the context to `Contexts` in [`src/schema.rs`](../src/schema.rs).
2. Update `CONTEXTS` constant in [`src/check.rs`](../src/check.rs) so it shows up in CLI help.
3. Implement detection logic:

   * Add a new function in `schema.rs` that sets the context and pushes `Evidence`.
   * Call it from `EnvSense::detect()`.
4. Write unit tests in `src/` and CLI tests in `tests/` to confirm detection.

---

## Adding a New Facet

1. Add the facet field to `Facets` in [`src/schema.rs`](../src/schema.rs).
2. Update `FACETS` constant in [`src/check.rs`](../src/check.rs).
3. Extend detection logic using declarative patterns:

   * For CI vendors, add `EnvMapping` rules in [`src/detectors/env_mapping.rs`](../src/detectors/env_mapping.rs).
   * For agents/editors, add `EnvMapping` rules in [`src/detectors/env_mapping.rs`](../src/detectors/env_mapping.rs).
   * For IDEs, add `EnvMapping` rules in [`src/detectors/env_mapping.rs`](../src/detectors/env_mapping.rs).
4. Add `Evidence` whenever a facet is set (handled automatically by declarative system).
5. Test via CLI:

   ```bash
   cargo run -- check facet:new_id=value
   ```

---

## Adding a New Trait

1. Add the trait field to `Traits` in [`src/schema.rs`](../src/schema.rs).
2. Update `TRAITS` constant in [`src/check.rs`](../src/check.rs).
3. Extend detection logic in:

   * [`src/detectors/terminal.rs`](../src/detectors/terminal.rs) for terminal properties.
   * Or add `ValueMapping` rules in the appropriate declarative detector for domain-specific traits.
4. Include evidence if not purely derived (handled automatically by declarative system).
5. Write CLI tests to validate `cargo run -- info --fields=traits`.

---

## Adding a New Agent

1. Add `EnvMapping` rules in [`src/detectors/env_mapping.rs`](../src/detectors/env_mapping.rs) for the new agent:

   ```rust
   EnvMapping {
       name: "new-agent".to_string(),
       indicators: vec![
           EnvIndicator::new("NEW_AGENT_VAR"),
       ],
       value_mappings: vec![
           ValueMapping {
               target_key: "agent_id".to_string(),
               source_key: "NEW_AGENT_ID".to_string(),
               transform: None,
               condition: None,
               validation_rules: vec![],
           },
       ],
   }
   ```

2. Add the agent to the appropriate detector's mapping list (e.g., `get_agent_mappings()`).
3. Test using shared test utilities:

   ```rust
   use envsense::detectors::test_utils::create_env_snapshot;
   
   #[test]
   fn test_new_agent_detection() {
       let snap = create_env_snapshot(vec![
           ("NEW_AGENT_VAR", "true"),
           ("NEW_AGENT_ID", "new-agent"),
       ]);
       
       let detector = DeclarativeAgentDetector;
       let detection = detector.detect(&snap);
       
       assert_eq!(detection.facets_patch.get("agent_id"), Some(&"new-agent".to_string()));
   }
   ```

---

## Adding a New CI Vendor

1. Add `EnvMapping` rules in [`src/detectors/env_mapping.rs`](../src/detectors/env_mapping.rs) for the new CI vendor:

   ```rust
   EnvMapping {
       name: "new-ci".to_string(),
       indicators: vec![
           EnvIndicator::new("NEW_CI_VAR"),
       ],
       value_mappings: vec![
           ValueMapping {
               target_key: "ci_vendor".to_string(),
               source_key: "NEW_CI_VENDOR".to_string(),
               transform: Some(ValueTransform::ToLowercase),
               condition: None,
               validation_rules: vec![],
           },
           ValueMapping {
               target_key: "branch".to_string(),
               source_key: "NEW_CI_BRANCH".to_string(),
               transform: None,
               condition: None,
               validation_rules: vec![ValidationRule::NotEmpty],
           },
       ],
   }
   ```

2. Add the CI vendor to `get_ci_mappings()` in [`src/detectors/env_mapping.rs`](../src/detectors/env_mapping.rs).
3. Add CLI test:

   ```bash
   NEW_CI_VAR=true NEW_CI_BRANCH=main cargo run -- check ci
   ```

---

## Adding a New Detector Module

For larger categories (e.g., containers, remote sessions):

1. Create a new module under `src/detectors/` following the declarative pattern.
2. Implement the `DeclarativeDetector` trait:

   ```rust
   pub struct DeclarativeContainerDetector;
   
   impl DeclarativeDetector for DeclarativeContainerDetector {
       type Facet = String; // or custom struct
       
       fn get_mappings() -> Vec<EnvMapping> {
           get_container_mappings()
       }
       
       fn create_detection() -> Detection<Self::Facet> {
           Detection::new()
       }
   }
   ```

3. Add `EnvMapping` rules in [`src/detectors/env_mapping.rs`](../src/detectors/env_mapping.rs).
4. Wire it into `EnvSense::detect()` in [`src/schema.rs`](../src/schema.rs).
5. Provide unit tests and CLI integration tests.

## Declarative Extension Patterns

### Adding Value Mappings

To extract specific values from environment variables:

```rust
ValueMapping {
    target_key: "custom_value".to_string(),
    source_key: "CUSTOM_ENV_VAR".to_string(),
    transform: Some(ValueTransform::ToLowercase),
    condition: Some(Condition::Exists("REQUIRED_VAR".to_string())),
    validation_rules: vec![ValidationRule::NotEmpty],
}
```

### Adding Conditional Logic

To apply mappings only under certain conditions:

```rust
ValueMapping {
    target_key: "is_feature_enabled".to_string(),
    source_key: "FEATURE_FLAG".to_string(),
    transform: Some(ValueTransform::ToBool),
    condition: Some(Condition::Equals("ENVIRONMENT".to_string(), "production".to_string())),
    validation_rules: vec![],
}
```

### Adding Transformations

Available transformations for value processing:

- `ValueTransform::ToBool` - Convert to boolean
- `ValueTransform::ToLowercase` - Convert to lowercase
- `ValueTransform::ToUppercase` - Convert to uppercase
- `ValueTransform::Trim` - Remove whitespace
- `ValueTransform::Replace` - String replacement
- `ValueTransform::Split` - Split into array
- `ValueTransform::Custom` - Custom transformation function

### Adding Validation Rules

Ensure extracted values meet requirements:

- `ValidationRule::NotEmpty` - Value must not be empty
- `ValidationRule::IsInteger` - Value must be an integer
- `ValidationRule::IsBoolean` - Value must be a boolean
- `ValidationRule::MatchesRegex` - Value must match regex pattern
- `ValidationRule::InRange` - Value must be within range
- `ValidationRule::AllowedValues` - Value must be in allowed list
- `ValidationRule::MinLength` - Minimum string length
- `ValidationRule::MaxLength` - Maximum string length

---

## CLI Integration

When adding new contexts/facets/traits:

* Verify `cargo run -- check --list` shows the new option.
* Ensure `cargo run -- check --explain` produces meaningful reasoning.
* Test with `cargo run -- info --json --fields=contexts,traits,facets` to verify output.

---

## Checklist for Every Extension

* [ ] Schema updated if new fields are introduced.
* [ ] Added to `CONTEXTS` / `FACETS` / `TRAITS` constants.
* [ ] Evidence added for every detection.
* [ ] Unit tests written for detection logic.
* [ ] CLI integration tests added.
* [ ] Documentation updated (README, `docs/`).