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
3. Extend detection logic:

   * For CI vendors, update `normalize_vendor()` in [`src/ci.rs`](../src/ci.rs).
   * For agents/editors, extend detection in [`src/agent.rs`](../src/agent.rs).
   * For IDEs, update `detect_ide()` in [`src/schema.rs`](../src/schema.rs).
4. Add `Evidence` whenever a facet is set.
5. Test via CLI:

   ```bash
   envsense check facet:new_id=value
   ```

---

## Adding a New Trait

1. Add the trait field to `Traits` in [`src/schema.rs`](../src/schema.rs).
2. Update `TRAITS` constant in [`src/check.rs`](../src/check.rs).
3. Extend detection logic in:

   * [`src/traits/terminal.rs`](../src/traits/terminal.rs) for terminal properties.
   * Or a new detector module if domain-specific.
4. Include evidence if not purely derived.
5. Write CLI tests to validate `envsense info --fields=traits`.

---

## Adding a New Agent

1. Update the `descriptor()` map in [`src/agent.rs`](../src/agent.rs) with:

   * Human name
   * Variant (e.g. `terminal`, `sandbox`)
   * Capabilities (`code-edit`, `run-commands`, etc.)
2. Add environment variable detection in `detect_agent()`.
3. Ensure secret-scrubbing rules in `is_secret()` apply to any new vars.
4. Test using a synthetic environment:

   ```rust
   let env = TestEnv { vars: hashmap!["NEW_AGENT" => "1"] };
   let det = detect_agent(&env);
   assert_eq!(det.agent.name, Some("new-agent".into()));
   ```

---

## Adding a New CI Vendor

1. Add normalization in `normalize_vendor()` in [`src/ci.rs`](../src/ci.rs).
2. Ensure both `vendor` (snake\_case ID) and `name` (human readable) are set.
3. Add CLI test:

   ```bash
   GITHUB_ACTIONS=1 envsense check ci
   ```

---

## Adding a New Detector Module

For larger categories (e.g., containers, remote sessions):

1. Create a new module under `src/`.
2. Define a struct for the facet(s).
3. Implement a `detect_*()` function.
4. Wire it into `EnvSense::detect()` in [`src/schema.rs`](../src/schema.rs).
5. Provide unit tests and CLI integration tests.

---

## CLI Integration

When adding new contexts/facets/traits:

* Verify `--list-checks` shows the new option.
* Ensure `--explain` produces meaningful `reason` and `signals`.

---

## Checklist for Every Extension

* [ ] Schema updated if new fields are introduced.
* [ ] Added to `CONTEXTS` / `FACETS` / `TRAITS` constants.
* [ ] Evidence added for every detection.
* [ ] Unit tests written for detection logic.
* [ ] CLI integration tests added.
* [ ] Documentation updated (README, `docs/`).