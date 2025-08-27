Here’s a concrete, no-fluff implementation plan that gets you from zero → Rust core → CLI → Node bindings. I’m biasing for things that reduce risk early (stable schema, deterministic behavior, testability).

# Phase 0 — Spec & Schema (lock this first)

**Goal:** a stable contract the CLI and bindings can share.

* **Result type (JSON & Rust struct)**

  ```ts
  type EnvSense = {
    contexts: { agent: boolean; ide: boolean; ci: boolean; container: boolean; remote: boolean; };
    facets: { agent_id?: string|null; ide_id?: string|null; ci_id?: string|null; container_id?: string|null; };
    traits: {
      is_interactive: boolean;
      is_tty_stdin: boolean; is_tty_stdout: boolean; is_tty_stderr: boolean;
      is_piped_stdin: boolean; is_piped_stdout: boolean;
      color_level: "none"|"basic"|"256"|"truecolor";
      supports_hyperlinks: boolean;
    };
    evidence: Array<{ signal: "env"|"tty"|"proc"|"fs"; key: string; value?: string; supports: string[]; confidence: number }>;
    version: string;          // schema version e.g. "0.1.0"
    rules_version: string;    // hash/semver of the ruleset bundled/loaded
  }
  ```

* **Principles**

  * Keep **contexts** (who/where) separate from **traits** (what it can do).
  * Every true/derived claim has **evidence** with a confidence.
  * Two-versioning model: `version` (schema) and `rules_version` (data).

* **Check expression grammar (v1)**

  * Start **simple, single predicate**:

    * `agent` (context)
    * `facet:ide_id=vscode`
    * `trait:is_interactive`
  * Defer boolean expressions (`A && B || !C`) to a later minor release.

# Phase 1 — Rust core library (`envsense-core`)

**Goal:** fast, dependency-light detector you can call from anywhere.

* **Crate layout**

  * `lib.rs` — `DetectOptions`, `DetectResult`, `detect()`.
  * `signals/` — low-level readers:

    * `env.rs` (env vars snapshot)
    * `tty.rs` (isatty per fd; use `is-terminal`)
    * `proc.rs` (optional parent process scan; behind feature flag `proc`)
    * `fs.rs` (cgroups/devcontainer markers; behind feature flag `containers`)
  * `detectors/` — pure functions that turn signals → claims:

    * `ide.rs`, `agent.rs`, `ci.rs`, `container.rs`, `remote.rs`, `terminal.rs`
  * `rules/` — built-in YAML (compiled in via `include_str!`) + loader for user rule packs.
  * `compose.rs` — precedence, scoring, and merging into `DetectResult`.
  * `check.rs` — parser/evaluator for the v1 single-predicate checks.

* **Options**

  ```rust
  pub struct DetectOptions {
      pub allow_proc_scan: bool,        // default false
      pub allow_fs_probe: bool,         // default true (read-only)
      pub user_rules_path: Option<PathBuf>,
      pub override_context: Option<String>, // e.g., "agent"
  }
  ```

* **Detections (bootstrap set)**

  * **IDE**: `TERM_PROGRAM=vscode` (+ version), `INSIDE_EMACS` (present)
  * **Agent**: `CURSOR_AGENT=1` (and allow user rules to add others)
  * **CI**: `CI=true` and vendor vars (`GITHUB_ACTIONS`, etc.) via rules
  * **Remote**: `SSH_CONNECTION|SSH_TTY|SSH_CLIENT` present
  * **Terminal traits**:

    * `is_*tty` via `is-terminal`
    * `is_interactive` from `$-` contains `i` (when visible) OR stdin+stdout TTY and not CI
    * `is_piped_*` = negation of isatty
    * `color_level` honoring `NO_COLOR`, `FORCE_COLOR` first; basic inference otherwise
    * `supports_hyperlinks` via known terminals + optional OSC8 probe (disabled by default)
  * **Container** (optional behind `containers`): cgroup/WSL/devcontainer markers

* **Confidence policy (initial)**

  * Explicit env from host (e.g., `TERM_PROGRAM=vscode`): **0.95**
  * Well-known CI vars: **0.9**
  * SSH envs: **0.9**
  * TTY checks: **0.9**
  * Heuristics (file markers/process ancestry): **0.6–0.8** (off by default where intrusive)

* **Testing**

  * **Snapshot tests**: synthetic env/TTY maps → golden JSON outputs.
  * **Property tests**: precedence holds (override > explicit > channel > ancestry > heuristics).
  * **Platform tests**: minimal matrix (Linux/macOS/Windows).
  * **No external processes**; pure, fast unit tests.

# Phase 2 — Rust FFI surface (`envsense-ffi`)

**Goal:** stable C ABI for bindings + reuse by CLI.

* Expose:

  * `envsense_detect_json(opts_json: *const c_char) -> *mut c_char` (caller frees)
  * `envsense_check(expr: *const c_char, opts_json: *const c_char) -> bool`
* Keep `opts_json` so non-Rust callers can pass feature toggles / rule paths.

# Phase 3 — CLI (`envsense`)

**Goal:** thin wrapper over the core/FFI; zero logic duplication.

* **Commands/flags**

  * Default: prints JSON (or pretty when `--pretty`)
  * `--json`, `--pretty`, `--only contexts|facets|traits`, `--explain`
  * `--check <predicate>` → exit 0/1, no output by default (use `-v` to print)
  * `--rules <path>` (user rule pack), `--allow-proc-scan`, `--deny-probe osc8`
  * `export --profile <name> --shell bash|zsh|fish` (optional later; uses profiles)

* **Behavior**

  * Respect `SHELL_CONTEXT_FORCE` as a hard override (logged in `evidence`).
  * `--explain` prints an evidence table: signal/key/value → claim → confidence.

* **DX niceties (later)**

  * Shell completions
  * `--rules-version` output

# Phase 4 — Node.js binding (`envsense-node`)

**Goal:** first-class Node API that mirrors the schema.

* **napi-rs** wrapper over `envsense-ffi`

  * `detect(options?: DetectOptions): Promise<EnvSense>`
  * `check(expr: string, options?: DetectOptions): Promise<boolean>`

* **Packaging**

  * Prebuilds for Linux/macOS/Windows (x64 + arm64) using GitHub Actions.
  * Fallback to local build if no prebuild matches.

* **TypeScript types** generated from the JSON schema.

* **Runtime ergonomics**

  * Respect `process.env` overrides automatically.
  * Don’t run OSC8 probe unless `options.allowProbes === true`.

# Phase 5 — Rules & Profiles (optional add-on)

**Goal:** keep detection (facts) separate from policy (what to set).

* **Rules**

  * Ship built-ins for common IDEs/CI/agents/terms.
  * Load `~/.config/envsense/rules.d/*.yml` (XDG on \*nix; `%AppData%` on Windows).
  * Validation + `rules_version` hashing.

* **Profiles**

  * TOML: map contexts/traits → env deltas.
  * Used only by `envsense export …`; not part of core detection.

# Acceptance criteria (per phase)

* **Core**

  * Deterministic outputs for a given env snapshot.
  * No syscalls that mutate state; read-only probes only.
  * <10ms typical detection on a warm process (no external commands).
* **CLI**

  * `envsense check agent` exit codes correct across platforms.
  * `--explain` always lists at least one evidence item for every `true` claim.
* **Node**

  * API stable; types shipped; works with ESM & CJS.
  * Prebuilds pass on supported OS/arch.

# Risk / reality check

* **Windows weirdness (ConPTY/ANSI):** lean on `is-terminal` and keep color detection conservative; let users override via `NO_COLOR/FORCE_COLOR`.
* **IDE/agent churn:** mitigate by **rules packs** (data) rather than releasing new binaries for every new vendor var.
* **Process ancestry privacy:** default **off**; opt-in via flag to avoid surprises.

# What to build first (practical ordering)

1. **Schema + core detectors** (env/tty/SSH/CI/VS Code + Cursor).
2. **Rust core crate** with tests and golden JSON.
3. **CLI** using the core; wire up `--check` and `--explain`.
4. **Node binding** via napi-rs with prebuild CI.
5. **Rules pack** for more vendors; optional profiles/export.

If you want, I can sketch the initial **Rust type definitions** and a **couple of detectors** (VS Code + agent + interactivity) as a starting code stub you can drop into a new repo.
