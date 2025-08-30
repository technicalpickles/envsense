
# Testing

`envsense` emphasizes **deterministic, auditable detection**. Tests ensure schema stability, correct detection logic, and consistent CLI behavior across environments.

---

## Testing Layers

### 1. Unit Tests

Each module has focused tests:

* **`src/detectors/agent_declarative.rs`**
  Tests declarative agent detection using `EnvMapping` patterns for Cursor, Cline, Aider, Replit, OpenHands, and override behavior (`ENVSENSE_AGENT`, `ENVSENSE_ASSUME_HUMAN`).
  Verifies correct agent detection, confidence scores, and facet assignment.

* **`src/detectors/ci_declarative.rs`**
  Tests declarative CI detection using `EnvMapping` patterns for GitHub Actions, GitLab CI, CircleCI, and other CI systems.
  Covers value extraction for branch names, PR status, and CI vendor detection.

* **`src/detectors/ide_declarative.rs`**
  Tests declarative IDE detection using `EnvMapping` patterns for VS Code, Cursor, and other development environments.

* **`src/detectors/env_mapping.rs`**
  Tests the core declarative detection system including `ValueMapping`, `ValueTransform`, `Condition`, and `ValidationRule` functionality.
  Covers conditional value extraction, transformations, and validation logic.

* **`src/detectors/terminal.rs`**
  Tests color level mapping (`none`, `ansi16`, `ansi256`, `truecolor`).
  Ensures derived fields (`is_piped_stdin`, `is_piped_stdout`) are correct.

* **`src/schema.rs`**
  Confirms schema version is serialized.
  Validates JSON schema generation with `schemars`.

* **`src/check.rs`**
  Validates check expression parsing: contexts, facets, traits, negation, invalid expressions.

---

### 2. CLI Tests

Located in `tests/cli.rs` and `tests/cli_terminal.rs`.

* **`info` command**

  * JSON output includes `schema_version`.
  * `--fields` limits output correctly.
  * `--raw` omits headings and colors.
  * Invalid fields produce exit code `2`.

* **`check` command**

  * Handles contexts, facets, traits correctly.
  * Honors `--any`/`--all` logic.
  * `--quiet` suppresses output.
  * Exit codes: `0` for success, `1` for failure, `2` for invalid input.
  * Special path for `check ci` prints human-readable messages.

* **Color detection**
  Validates that TTY detection and `NO_COLOR` override colorized output.

* **Meta fields**
  Ensures `meta` includes `schema_version`.

* **Terminal traits**
  `tests/cli_terminal.rs` asserts correct piped/TTY trait reporting.

---

### 3. Cross-Platform Matrix

The CI pipeline runs `cargo test --all --locked` on **Linux** and **macOS**.
Windows support is expected but may require extra care around TTY detection and ANSI color handling.

---

## Guidelines for Adding Tests

1. **Write unit tests** alongside new detectors to lock in expected behavior for known env vars and edge cases.
2. **Add CLI integration tests** in `tests/` when extending CLI options or output formats.
3. **Use synthetic environments** (`temp-env`, `script`) to simulate different runtime conditions.
4. **Assert schema stability** when modifying `EnvSense` or evidence structures:

   * Bump `SCHEMA_VERSION`.
   * Update golden tests.
   * Ensure `json_schema_generates` passes.
5. **Test precedence rules** (override > explicit > channel > ancestry > heuristics) whenever new detection signals are introduced.

## Declarative Testing Patterns

When testing declarative detection patterns, follow these guidelines:

### Testing EnvMapping Rules
```rust
#[test]
fn test_github_actions_mapping() {
    let snap = create_env_snapshot(vec![
        ("GITHUB_ACTIONS", "true"),
        ("GITHUB_REF_NAME", "main"),
    ]);
    
    let detector = DeclarativeCiDetector;
    let detection = detector.detect(&snap);
    
    assert_eq!(detection.contexts_add, vec!["ci"]);
    assert_eq!(detection.facets_patch.get("ci_id"), Some(&"github_actions".to_string()));
}
```

### Testing Value Mappings
```rust
#[test]
fn test_branch_extraction() {
    let snap = create_env_snapshot(vec![
        ("GITHUB_ACTIONS", "true"),
        ("GITHUB_REF_NAME", "feature-branch"),
    ]);
    
    let mappings = get_ci_mappings();
    let github_mapping = mappings.iter().find(|m| m.name == "github-actions").unwrap();
    let extracted = github_mapping.extract_values(&snap.env_vars);
    
    assert_eq!(extracted.get("branch"), Some(&"feature-branch".to_string()));
}
```

### Testing Conditional Logic
```rust
#[test]
fn test_conditional_value_mapping() {
    let snap = create_env_snapshot(vec![
        ("GITHUB_ACTIONS", "true"),
        ("GITHUB_EVENT_NAME", "pull_request"),
    ]);
    
    // Test that PR status is extracted when in GitHub Actions
    let mappings = get_ci_mappings();
    let github_mapping = mappings.iter().find(|m| m.name == "github-actions").unwrap();
    let extracted = github_mapping.extract_values(&snap.env_vars);
    
    assert_eq!(extracted.get("is_pr"), Some(&true));
}
```

---

## Test Tools & Crates

* [`assert_cmd`](https://crates.io/crates/assert_cmd) – run compiled CLI and assert outputs/exit codes.
* [`predicates`](https://crates.io/crates/predicates) – check stdout/stderr content.
* [`serial_test`](https://crates.io/crates/serial_test) – isolate env-var-sensitive tests.
* [`temp-env`](https://crates.io/crates/temp-env) – temporary environment manipulation.
* [`schemars`](https://crates.io/crates/schemars) – schema generation validation.
* [`insta`](https://crates.io/crates/insta) – snapshot testing for JSON outputs.

---

### 4. Shared Test Utilities

Located in `src/detectors/test_utils.rs`, these utilities provide standardized testing patterns across all detector modules.

* **Purpose**: Centralize common testing patterns and reduce code duplication
* **Functions**:
  - `create_env_snapshot(env_vars: Vec<(&str, &str)>) -> EnvSnapshot` - Create basic environment snapshots for testing
  - `create_env_snapshot_with_tty(env_vars: Vec<(&str, &str)>, tty_stdin: bool, tty_stdout: bool, tty_stderr: bool) -> EnvSnapshot` - Create snapshots with custom TTY settings

* **Usage Example**:
  ```rust
  use envsense::detectors::test_utils::create_env_snapshot;
  
  #[test]
  fn test_agent_detection() {
      let snap = create_env_snapshot(vec![
          ("TERM_PROGRAM", "cursor"),
          ("ENVSENSE_AGENT", "cursor"),
      ]);
      // ... test logic
  }
  ```

* **Benefits**:
  - **Consistency**: All tests use the same snapshot creation patterns
  - **Maintainability**: Changes to snapshot creation only need to be made in one place
  - **Readability**: Clear, standardized test setup across all modules

### 5. Snapshot Tests

Located in `tests/info_snapshots.rs`, these tests validate that the CLI produces consistent JSON output across different environments.

* **Purpose**: Ensures detection output remains stable and predictable
* **Coverage**: Tests various environments (VS Code, Cursor, CI systems, terminals, etc.)
* **Files**: 
  - `tests/snapshots/*.snap` - Insta snapshot files
  - `tests/snapshots/*.json` - Golden JSON outputs

* **Updating Snapshots**: When schema changes (like removing `rules_version`):
  ```bash
  # Run tests to see failures
  cargo test --test info_snapshots
  
  # Install cargo-insta if not already installed
  cargo install cargo-insta
  
  # Accept changes (non-interactive)
  cargo insta accept
  ```

* **Schema Changes**: When making breaking schema changes:
  1. Bump `SCHEMA_VERSION` in `src/schema.rs`
  2. Update all snapshot tests to expect new version
  3. Run `cargo insta accept` to update snapshots
  4. Verify all tests pass

---

## Invariants to Maintain

* Schema version is always serialized.
* Every `true` detection has at least one supporting `Evidence` item (test with `--explain`).
* CLI output must remain stable and scriptable:

  * Human-readable modes (`pretty`, `raw`) tested for formatting.
  * JSON mode tested for structure and fields.