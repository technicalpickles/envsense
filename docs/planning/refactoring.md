Thanks for the link. From the shared history, it looks like the prior refactor attempted a big-bang rewrite (about **+703 / −1490** LOC), which likely dropped edge-cases and compatibility paths while moving code around. That’s the classic way functionality “disappears” during a refactor. ([ChatGPT][1])

Here’s a tighter, **functionality-preserving** implementation plan you can hand to an agent (or follow yourself). It’s designed to move toward `ide.rs` + unified detectors **without** deleting working logic until we’ve proven parity.

# Guardrails (non‑negotiable)

* **No behavior change until parity proven.** Introduce new code behind shims/flags; keep legacy paths callable.
* **Golden‑master snapshots** for CLI JSON before touching code.
* **Refactor = move + re‑export; not delete.** Any removed function must have an equivalent shim or explicit test coverage proving it’s unused.
* **Small, verifiable PRs** with explicit “exit criteria” and a rollback plan.

# Preparatory work (PR 0)

1. **Freeze a baseline**

   * Tag a release (e.g., `v0.0-baseline`).
   * Capture **JSON snapshots** for representative environments:

     * VS Code, VS Code Insiders, Cursor
     * GitHub Actions, GitLab CI
     * tmux/screen, plain TTY, piped I/O
     * Popular shells (bash/zsh), Mac/Linux
   * Tooling:

     * Use `assert_cmd` + `insta` (or `similar-asserts`) to snapshot `envsense info --json`.
     * Write small fixture runners that inject env maps (no real processes needed).

2. **Document invariants**

   * CONTRACT.md listing: field names (snake\_case), enum strings, precedence order (user override > explicit signals > channel > ancestry > heuristics), and existing evidence cues.
   * Any consumer‑visible JSON key/enum must remain stable (or have serde aliases).

# Stepwise refactor plan

## PR 1 — Test harness & snapshots (no code moves)

**Goal:** You can prove when you break something.

* Add `tests/snapshots/` with fixtures per environment.
* Add a script: `scripts/compare-baseline.sh` that runs current binary and compares output to snapshots (use `jq -S` + `diff`).
* CI: run snapshot tests on PRs; fail on drift.

**Exit criteria:** Green CI; snapshots established.

---

## PR 2 — Extract **types‑only** `schema.rs` (mechanical)

**Goal:** Separate *data* from *logic* without changing behavior.

* Move pure types (Report, ContextKind, Traits, Facets, Meta, enums) into `schema.rs`.
* Keep serde config: `#[serde(rename_all = "snake_case")]` and `#[serde(alias = "...")]` for any legacy names.
* No logic moves yet. All existing functions continue to compile.

**Exit criteria:** 0 snapshot diffs.

---

## PR 3 — Introduce `detectors/` and **Detector** trait (no switching yet)

**Goal:** Create the new extension point without using it in production.

* New files:

  * `src/detectors/mod.rs` with:

    ```rust
    pub trait Detector {
        fn name(&self) -> &'static str;
        fn detect(&self, snap: &EnvSnapshot) -> Detection;
    }
    pub struct Detection { /* contexts_add, traits_patch, facets_patch, evidence, confidence */ }
    pub struct EnvSnapshot { /* env map, tty info, proc hints */ }
    ```
  * `src/evidence.rs` with `EvidenceItem` + `EvidenceSource`.
  * `src/engine.rs` that can run detectors and merge results, but **isn’t wired** to CLI yet.
* Do not delete old logic; do not change CLI code path.

**Exit criteria:** 0 snapshot diffs.

---

## PR 4 — Extract `ide.rs` (logic move with **shim**)

**Goal:** Move editor detection out of `schema.rs` with zero behavior change.

* Create `src/detectors/ide.rs` and **lift** the exact IDE/editor heuristics (VS Code, Insiders, Cursor, etc.).
* Provide a **shim function** that matches the old call site signature; the shim calls `IdeDetector.detect(..)` and converts the `Detection` back into the legacy structures the caller expects.
* Keep all env var checks intact; don’t rename keys/strings.
* Add targeted unit tests for `IdeDetector` using env maps.

**Exit criteria:** 0 snapshot diffs; unit tests cover VS Code, Insiders, Cursor envs.

---

## PR 5 — Adapt `agent.rs` and `ci.rs` to **Detector** (still behind shim)

**Goal:** Prepare other detectors for the new engine.

* Wrap existing `agent` and `ci` detection with Detector adapters returning `Detection`.
* Keep legacy entry points; adapters are called from shims so the CLI path is unchanged.

**Exit criteria:** 0 snapshot diffs.

---

## PR 6 — Wire **engine** behind a feature flag (dual‑run compare)

**Goal:** Prove parity before switching.

* Add flag/env: `ENVSENSE_EXPERIMENTAL_ENGINE=1`.
* When enabled, run **both**: legacy pipeline and new engine; compare `Report` (after normalizing order, defaults).
* Log `"DIFF:"` with a structured diff if mismatched (dev‑only).
* Add a test that enables the flag; assert equality across your snapshot fixtures.

**Exit criteria:** Flagged runs are equal on CI; no user‑visible changes.

---

## PR 7 — Flip default to new engine; keep fallback

**Goal:** Cut over safely.

* Default to new engine; allow `ENVSENSE_LEGACY_ENGINE=1` to revert.
* Keep the legacy code paths in tree for one release cycle.

**Exit criteria:** 0 snapshot diffs; release notes include fallback instructions.

---

## PR 8 — Cleanup (post‑parity)

**Goal:** Remove dead code with confidence.

* After one or two releases, delete legacy paths once telemetry/snapshots show stable parity.
* Keep serde aliases for any public enums/keys that were renamed internally.

**Exit criteria:** No missed functionality in tests; coverage unchanged or improved.

# Evidence & debugging aids (add early)

* `--debug-detectors` flag that prints:

  * Each detector’s proposed contexts/facets/traits and **confidence**.
  * The **evidence** items (source/key/value) used to reach a conclusion.
* `--explain-precedence` flag to show why conflicts (e.g., two `ide_id`s) resolved a certain way.
* Deterministic merges (sort keys, stable ordering) so snapshots don’t flap.

# Acceptance checks per PR

* ✅ **No snapshot diffs** (unless a PR is explicitly about behavior change).
* ✅ All detectors still fire in the same situations (IDE, CI, agent).
* ✅ Precedence order preserved: user override > explicit signals > channel > ancestry > heuristics.
* ✅ JSON keys stay snake\_case and stable; add `#[serde(alias)]` when moving/renaming.
* ✅ Tests: unit (detectors), integration (CLI), conflict cases (e.g., VS Code + Cursor ancestry), and negative cases.

# “Agent‑proof” prompt (for your coding agent)

Paste this at the top of any agent run to keep it on rails:

> **You are performing a refactor that must preserve behavior.**
>
> * Do not delete logic or tests. You may **move** code and add shims.
> * Maintain JSON output **identical** to baseline snapshots (use `insta`).
> * Add new files/modules (`schema.rs`, `detectors/*`, `engine.rs`, `evidence.rs`), but keep old entry points calling into shims until parity is proven.
> * Keep all env var heuristics and precedence **unchanged**.
> * If you must rename, add `#[serde(alias = "...")]` to preserve external strings.
> * Every PR must pass: (1) unit tests, (2) golden snapshot tests, (3) no CLI output diffs.
> * If unsure, prefer duplication over deletion; clean up only after the engine is default and parity is verified.

---

If you want, I can turn this into a set of ready-to-run GitHub issue checklists (one per PR) plus a tiny Rust test scaffold (`assert_cmd` + `insta`) you can drop into `tests/`. The main thing we’ll avoid this time is another “−1490 LOC” leap without a safety net. ([ChatGPT][1])

[1]: https://chatgpt.com/s/cd_68a9acdcc61c8191b5ccee9b35ce6abd "ChatGPT - Shared Content"
