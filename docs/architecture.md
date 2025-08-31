# Architecture

`envsense` is a Rust library and CLI for detecting runtime environments and
exposing them in a structured way. It provides a stable JSON schema so other
languages (via FFI and bindings) can consume the same results.

---

## High-Level Flow

```
 ┌───────────┐     ┌───────────┐     ┌────────────┐     ┌───────────┐
 │  Signals  │──▶──│ Detectors │──▶──│  Evidence  │──▶──│ EnvSense  │
 └───────────┘     └───────────┘     └────────────┘     └───────────┘
                        │                               │
                        ▼                               ▼
                   Contexts, Facets, Traits         CLI / Bindings
```

1. **Signals** – Low-level observations from the host environment:
   - Environment variables
   - TTY status
   - Optional process ancestry
   - Filesystem markers

2. **Detectors** – Functions that interpret signals for a specific domain (CI,
   agent, IDE, terminal).

3. **Evidence** – Each claim is backed by a `signal`, `key`, optional `value`,
   list of `supports`, and a `confidence` score.

4. **EnvSense struct** – Composed result that includes:
   - **Contexts** (broad categories like `agent`, `ide`, `ci`)
   - **Facets** (specific identifiers like `ci_id=github_actions`)
   - **Traits** (capabilities like `supports_hyperlinks`)
   - **Evidence** (audit trail)
   - **Schema version / rules version**

---

## Core Concepts

- **Contexts**: high-level categories (agent, ide, ci, container, remote).
- **Facets**: finer identifiers (e.g., `cursor`, `vscode-insiders`,
  `github_actions`).
- **Traits**: booleans/properties about terminal and session (TTYs, color
  depth).
- **Evidence**: why something was set, traceable to a signal.

## Declarative Detection System

The core of envsense uses a declarative pattern-based detection system:

### EnvMapping

Each detection domain (CI, agent, IDE) defines `EnvMapping` rules that specify:

- **Indicators**: Environment variables that indicate the presence of an
  environment
- **Value Mappings**: How to extract specific values (branch names, PR status,
  etc.)
- **Conditions**: When certain mappings should be applied
- **Transformations**: How to process extracted values

### ValueMapping

Extracts specific values from environment variables:

```rust
ValueMapping {
    target_key: "branch".to_string(),
    source_key: "GITHUB_REF_NAME".to_string(),
    transform: Some(ValueTransform::ToLowercase),
    condition: Some(Condition::Exists("GITHUB_ACTIONS".to_string())),
    validation_rules: vec![ValidationRule::NotEmpty],
}
```

### Benefits

- **Consistency**: All detection follows the same pattern
- **Extensibility**: Easy to add new environments and value extractions
- **Maintainability**: Centralized logic reduces duplication
- **Testability**: Declarative patterns are easier to test

---

## Detection Modules

- [`src/detectors/agent_declarative.rs`](../src/detectors/agent_declarative.rs)
  Detects AI coding agents using declarative patterns. Handles overrides
  (`ENVSENSE_AGENT`, `ENVSENSE_ASSUME_HUMAN`) and secret-safe session capture.

- [`src/detectors/ci_declarative.rs`](../src/detectors/ci_declarative.rs)
  Detects CI environments using declarative patterns. Supports GitHub Actions,
  GitLab CI, CircleCI, and other CI systems with value extraction for branch
  names, PR status, and more.

- [`src/detectors/ide_declarative.rs`](../src/detectors/ide_declarative.rs)
  Detects IDE environments using declarative patterns. Identifies VS Code,
  Cursor, and other development environments.

- [`src/detectors/env_mapping.rs`](../src/detectors/env_mapping.rs) Core
  declarative detection system. Defines `EnvMapping`, `ValueMapping`, and
  related structures for environment detection patterns.

- [`src/detectors/terminal.rs`](../src/detectors/terminal.rs) Detects TTYs,
  interactivity, color depth (`supports-color`), and OSC-8 hyperlink support.

- [`src/schema.rs`](../src/schema.rs) Defines the `EnvSense` struct, JSON
  schema, and orchestrates the detection pipeline using the declarative system.

- [`src/check.rs`](../src/check.rs) Implements the simple check expression
  grammar:
  - `context`
  - `facet:key=value`
  - `trait:key` with optional `!` negation.

- [`src/main.rs`](../src/main.rs) CLI entry point using `clap`. Provides:
  - `info` (summary, JSON, raw/plain/pretty)
  - `check` (evaluate predicates with `--any`, `--all`, `--quiet`, `--format`,
    `--explain`)

---

## Precedence Rules

Detection precedence ensures consistent results:

1. **User overrides** (`ENVSENSE_AGENT`, `ENVSENSE_ASSUME_HUMAN`)
2. **Explicit environment variables** (e.g. `TERM_PROGRAM`, `CI`)
3. **Execution channel** (SSH markers, container markers)
4. **Process ancestry** (opt-in)
5. **Heuristics** (e.g. file paths)

---

## Schema Versioning

- The schema is versioned via `SCHEMA_VERSION` (`0.2.0` currently).
- Every detection result includes `version` (schema).
- Schema evolution requires bumping `version` and updating tests.
- Recent changes: CI detection moved from nested `ci` facet to flat traits
  structure.

---

## Evidence Model

Each `Evidence` entry contains:

- `signal` – source of information (`Env`, `Tty`, `Proc`, `Fs`)
- `key` – the specific variable or probe name
- `value` – optional captured value
- `supports` – contexts/facets/traits it backs
- `confidence` – float in `[0.0, 1.0]`

This allows `--explain` to surface reasoning for any true claim.

---

## Outputs

- **Human summary** (`envsense info`)
- **Stable JSON** (contract for bindings)
- **Check predicates** (`envsense check`) with plain/pretty/json output
- **Exit codes** for scripting

---

## Extensibility

- Adding a new detection requires:
  1. **Declarative approach**: Define `EnvMapping` rules in the appropriate
     detector module.
  2. **Value extraction**: Add `ValueMapping` rules for extracting specific
     values.
  3. **Schema updates**: Add new facets/traits to `src/schema.rs` if needed.
  4. **CLI exposure**: Extend `CONTEXTS`, `FACETS`, or `TRAITS` arrays in
     `check.rs`.
  5. **Testing**: Write tests in `tests/` for CLI and golden JSON.

### Adding New CI Vendors

```rust
// In src/detectors/env_mapping.rs
pub fn get_ci_mappings() -> Vec<EnvMapping> {
    vec![
        // ... existing mappings
        EnvMapping {
            name: "my-ci".to_string(),
            indicators: vec![
                EnvIndicator::new("MY_CI_VAR"),
            ],
            value_mappings: vec![
                ValueMapping {
                    target_key: "branch".to_string(),
                    source_key: "MY_CI_BRANCH".to_string(),
                    transform: None,
                    condition: None,
                    validation_rules: vec![],
                },
            ],
        },
    ]
}
```

---

## Language Bindings

- The Rust core is designed to expose a C-ABI (`envsense-ffi`).
- Node.js binding is first-class via `napi-rs`, exposing `detect()` and
  `check()` with TypeScript types generated from the JSON schema.
