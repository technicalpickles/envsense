# envsense

**envsense** is a cross-language library and CLI for detecting the runtime environment and exposing it in a structured way. It helps tools and scripts adapt their behavior based on where they are running.

## Motivation

Most developers end up writing brittle ad-hoc checks in their shell configs or tools:

* *"If I’m in VS Code, set `EDITOR=code -w`"*
* *"If stdout is piped, disable color and paging"*
* *"If running in a coding agent, simplify my prompt"*

These heuristics get duplicated across dotfiles and codebases. **envsense** centralizes and standardizes this detection.

---

## Quick Start (Shell)

```bash
# Detect and show agent status
envsense check agent
envsense check --json agent

# Simple check for any coding agent
envsense check agent && echo "Running inside a coding agent"

# Check if specifically running in Cursor
envsense check facet:agent_id=cursor && echo "Cursor detected"

# Check if running in VS Code or VS Code Insiders
envsense check facet:ide_id=vscode && echo "VS Code"
envsense check facet:ide_id=vscode-insiders && echo "VS Code Insiders"

# Combine checks in scripts
if envsense check -q facet:ide_id=cursor; then
  export PAGER=cat
fi

# Check multiple conditions (any must match)
envsense check --any agent ide && echo "Running in agent or IDE"

# Check multiple conditions (all must match)
envsense check --all agent trait:is_interactive && echo "Interactive agent session"

# Get detailed reasoning for checks
envsense check --explain facet:ide_id=cursor

# List all available predicates
envsense check --list
```

---

## Exploring the Environment

You can also inspect the full structured output to see what envsense detects:

```bash
# Human-friendly summary
envsense info

# Print detected environment as JSON
envsense info --json

# Restrict to specific keys
envsense info --fields contexts,traits

# Pipe-friendly text
envsense info --raw

# Disable color output
envsense info --no-color
```

Example JSON output:

```json
{
  "contexts": [
    "agent",
    "ide"
  ],
  "traits": {
    "is_interactive": true,
    "is_tty_stdin": true,
    "is_tty_stdout": true,
    "is_tty_stderr": true,
    "is_piped_stdin": false,
    "is_piped_stdout": false,
    "color_level": "truecolor",
    "supports_hyperlinks": true
  },
  "facets": {
    "agent_id": "cursor",
    "ide_id": "cursor"
  },
  "evidence": [],
  "version": "0.2.0"
}
```

## Command Line Options

### Check Command Options

The `check` command evaluates predicates against the environment and exits with status 0 on success, 1 on failure.

#### Output Control
- `--json` - Output results as JSON (stable schema)
- `-q, --quiet` - Suppress output (useful in scripts)
- `--explain` - Show reasoning for each check result

#### Evaluation Modes
- `--any` - Succeed if any predicate matches (default: all must match)
- `--all` - Require all predicates to match (default behavior)

#### Discovery
- `--list` - List all available predicates

#### Examples
```bash
# Basic checks
envsense check agent                    # Exit 0 if agent detected, 1 otherwise
envsense check -q agent                # Silent check (no output)
envsense check --json agent            # JSON output with result

# Multiple predicates
envsense check agent ide               # Both must match (AND logic)
envsense check --any agent ide         # Either can match (OR logic)
envsense check --all agent ide         # Both must match (explicit AND)

# Get reasoning
envsense check --explain agent         # Shows why agent was/wasn't detected
envsense check --json --explain agent  # JSON with reasoning included

# List available predicates
envsense check --list                  # Shows all contexts, facets, and traits
```

### Info Command Options

The `info` command shows detailed environment information.

#### Output Formats
- `--json` - Output as JSON (stable schema)
- `--raw` - Plain text without colors or headers (pipe-friendly)
- `--no-color` - Disable color output

#### Field Selection
- `--fields <list>` - Comma-separated keys to include: `contexts`, `traits`, `facets`, `meta`

#### Examples
```bash
# Different output formats
envsense info                          # Human-friendly with colors
envsense info --json                   # JSON output
envsense info --raw                    # Plain text, no formatting
envsense info --no-color               # Human-friendly, no colors

# Field filtering
envsense info --fields contexts,traits # Only contexts and traits
envsense info --json --fields facets   # JSON with only facets
envsense info --raw --fields meta      # Raw output with only metadata
```

### Global Options

- `--no-color` - Disable color output (works on all commands)

### Exit Codes

- **0** - Success (predicates matched as expected)
- **1** - Failure (predicates did not match)
- **2** - Error (invalid arguments, parsing errors)

#### Exit Code Examples
```bash
envsense check agent && echo "Agent detected"     # Only runs if agent=true
envsense check !agent && echo "Not in agent"      # Only runs if agent=false
envsense check --any agent ide || echo "Neither"  # Runs if neither matches
```

## Key Concepts

* **Contexts** — broad categories of environment (`agent`, `ide`, `ci`, `container`, `remote`).
* **Facets** — more specific identifiers (`agent_id=cursor`, `ide_id=vscode`).
* **Traits** — capabilities or properties (`is_interactive`, `supports_hyperlinks`, `color_level`).
* **Evidence** — why envsense believes something (env vars, TTY checks, etc.), with confidence scores.

### Ask: Which category is it?

* *“Running in VS Code”* → **Context/Facet** (`ide`, `ide_id=vscode`).
* *“Running in GitHub Actions”* → **Facet** (`ci_id=github`).
* *“Running in CI at all”* → **Context** (`ci=true`).
* *“Can print hyperlinks”* → **Trait** (`supports_hyperlinks=true`).
* *“stdout is not a TTY”* → **Trait** (`is_interactive=false`).

### Snippet Examples

```bash
# Context: any CI
envsense check ci && echo "In CI"

# Facet: specifically GitHub Actions
envsense check facet:ci_id=github && echo "In GitHub Actions"

# Trait: non-interactive
if ! envsense check trait:is_interactive; then
  echo "Running non-interactive"
fi
```

### Predicate Syntax

Predicates follow a simple syntax pattern:

```bash
# Contexts (broad categories)
envsense check agent
envsense check ide
envsense check ci
envsense check container
envsense check remote

# Facets (specific identifiers)
envsense check facet:agent_id=cursor
envsense check facet:ide_id=vscode
envsense check facet:ci_id=github
envsense check facet:container_id=docker

# Traits (capabilities/properties)
envsense check trait:is_interactive
envsense check trait:supports_hyperlinks
envsense check trait:is_ci

# Negation (prefix with !)
envsense check !agent
envsense check !trait:is_interactive
envsense check !facet:ide_id=vscode
```

### CI detection

Detect if envsense is running in a CI environment and inspect details:

```bash
# Human-friendly summary
envsense info --fields=facets --no-color

# JSON output
envsense info --json --fields=traits,facets

# Simple check that exits 0 on CI, 1 otherwise
envsense check ci
```

Sample `cargo run -- info --fields=facets --no-color` output on GitHub Actions:

```
Facets:
  ci_id = github_actions
```

And the corresponding JSON fragment:

```json
{
  "traits": {
    "is_ci": true,
    "ci_vendor": "github_actions",
    "ci_name": "GitHub Actions",
    "is_pr": true,
    "branch": "main"
  },
  "facets": {
    "ci_id": "github_actions"
  }
}
```

---

## Language Bindings

* **Rust** (Primary):

  ```rust
  use envsense::detect_environment;
  
  let env = detect_environment();
  if env.contexts.agent {
      println!("Agent detected");
  }
  if env.facets.ide_id.as_deref() == Some("cursor") {
      println!("Cursor detected");
  }
  if !env.traits.is_interactive {
      println!("Non-interactive session");
  }
  ```

* **Node.js** (Planned):

  ```js
  import { detect } from "envsense";
  const ctx = await detect();
  if (ctx.contexts.agent) console.log("Agent detected");
  if (ctx.facets.ide_id === "cursor") console.log("Cursor detected");
  if (!ctx.traits.is_interactive) console.log("Non-interactive session");
  ```

---

## Detection Strategy

1. **Explicit signals** — documented env vars (e.g. `TERM_PROGRAM`, `INSIDE_EMACS`, `CI`).
2. **Execution channel** — SSH env vars, container cgroups, devcontainer markers.
3. **Process ancestry** — parent process names (optional, behind a flag).
4. **Heuristics** — last resort (file paths, working dir markers).

Precedence is: user override > explicit > channel > ancestry > heuristics.

---

## Terminal Features Detected

* **Interactivity**

  * Shell flags (interactive mode)
  * TTY checks for stdin/stdout/stderr
  * Pipe/redirect detection
* **Colors**

  * Honors `NO_COLOR`, `FORCE_COLOR`
  * Detects depth: none, basic, 256, truecolor
* **Hyperlinks (OSC 8)**

  * Known supporting terminals (iTerm2, kitty, WezTerm, VS Code, etc.)
  * Optional probe for fallback

---

## Project Status

* **Rust implementation complete** - Core library and CLI fully functional
* **Declarative detection system** - Environment detection using declarative patterns
* **Comprehensive CI support** - GitHub Actions, GitLab CI, CircleCI, and more
* **Extensible architecture** - Easy to add new detection patterns
* **Production ready** - Used in real-world environments

---

## Roadmap

* [x] Implement baseline detectors (env vars, TTY)
* [x] CLI output in JSON + pretty modes
* [x] Rule engine for contexts/facets/traits
* [x] Declarative detection system
* [x] Comprehensive CI environment support
* [ ] Node binding via napi-rs
* [ ] Additional language bindings
* [ ] Advanced value extraction patterns

---

## Development

### Prerequisites

- Rust 1.70+
- `cargo-insta` for snapshot testing: `cargo install cargo-insta`

### Testing

```bash
# Run all tests
cargo test --all

# Run snapshot tests
cargo test --test info_snapshots

# Update snapshots after schema changes
cargo insta accept

# Test specific components
cargo test --package envsense
cargo test --package envsense-macros
```

### Schema Changes

When making breaking schema changes (like removing fields):

1. Bump `SCHEMA_VERSION` in `src/schema.rs`
2. Update tests to expect new version
3. Run `cargo insta accept` to update snapshots
4. Verify all tests pass

### Development Workflow

```bash
# Format code (enforced by pre-commit hooks)
cargo fmt --all

# Lint and fix issues
cargo clippy --all --fix -D warnings

# Run full test suite
cargo test --all

# Build release version
cargo build --release
```

See `docs/testing.md` for detailed testing guidelines.

---

## License

MIT

---

**envsense**: environment awareness for any tool, in any language.