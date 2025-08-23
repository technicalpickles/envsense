# Architecture

`envsense` is a Rust library and CLI for detecting runtime environments and exposing them in a structured way. It provides a stable JSON schema so other languages (via FFI and bindings) can consume the same results.

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

   * Environment variables
   * TTY status
   * Optional process ancestry
   * Filesystem markers

2. **Detectors** – Functions that interpret signals for a specific domain (CI, agent, IDE, terminal).

3. **Evidence** – Each claim is backed by a `signal`, `key`, optional `value`, list of `supports`, and a `confidence` score.

4. **EnvSense struct** – Composed result that includes:

   * **Contexts** (broad categories like `agent`, `ide`, `ci`)
   * **Facets** (specific identifiers like `ci_id=github_actions`)
   * **Traits** (capabilities like `supports_hyperlinks`)
   * **Evidence** (audit trail)
   * **Schema version / rules version**

---

## Core Concepts

* **Contexts**: high-level categories (agent, ide, ci, container, remote).
* **Facets**: finer identifiers (e.g., `cursor`, `vscode-insiders`, `github_actions`).
* **Traits**: booleans/properties about terminal and session (TTYs, color depth).
* **Evidence**: why something was set, traceable to a signal.

---

## Detection Modules

* [`src/agent.rs`](../src/agent.rs)
  Detects AI coding agents and related facets (editor, host). Handles overrides (`ENVSENSE_AGENT`, `ENVSENSE_ASSUME_HUMAN`) and secret-safe session capture.

* [`src/ci.rs`](../src/ci.rs)
  Wraps [`ci_info`](https://crates.io/crates/ci_info) to detect CI vendors and normalize them into consistent identifiers.

* [`src/traits/terminal.rs`](../src/traits/terminal.rs)
  Detects TTYs, interactivity, color depth (`supports-color`), and OSC-8 hyperlink support.

* [`src/schema.rs`](../src/schema.rs)
  Defines the `EnvSense` struct, JSON schema, and orchestrates the detection pipeline (terminal → agent → CI → IDE).

* [`src/check.rs`](../src/check.rs)
  Implements the simple check expression grammar:

  * `context`
  * `facet:key=value`
  * `trait:key`
    with optional `!` negation.

* [`src/main.rs`](../src/main.rs)
  CLI entry point using `clap`. Provides:

  * `info` (summary, JSON, raw/plain/pretty)
  * `check` (evaluate predicates with `--any`, `--all`, `--quiet`, `--format`, `--explain`)

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

* The schema is versioned via `SCHEMA_VERSION` (`0.1.0` currently).
* Every detection result includes `version` (schema) and `rules_version` (ruleset hash/semver).
* Schema evolution requires bumping `version` and updating tests.

---

## Evidence Model

Each `Evidence` entry contains:

* `signal` – source of information (`Env`, `Tty`, `Proc`, `Fs`)
* `key` – the specific variable or probe name
* `value` – optional captured value
* `supports` – contexts/facets/traits it backs
* `confidence` – float in `[0.0, 1.0]`

This allows `--explain` to surface reasoning for any true claim.

---

## Outputs

* **Human summary** (`envsense info`)
* **Stable JSON** (contract for bindings)
* **Check predicates** (`envsense check`) with plain/pretty/json output
* **Exit codes** for scripting

---

## Extensibility

* Adding a new detection requires:

  1. Updating detector logic (new env vars or rules).
  2. Adding schema facets/traits if needed.
  3. Extending `CONTEXTS`, `FACETS`, or `TRAITS` arrays in `check.rs` to expose in help.
  4. Writing tests in `tests/` for CLI and golden JSON.

---

## Language Bindings

* The Rust core is designed to expose a C-ABI (`envsense-ffi`).
* Node.js binding is first-class via `napi-rs`, exposing `detect()` and `check()` with TypeScript types generated from the JSON schema.