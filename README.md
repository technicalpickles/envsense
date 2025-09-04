# envsense

**envsense** is a cross-language library and CLI for detecting the runtime
environment and exposing it in a structured way. It helps tools and scripts
adapt their behavior based on where they are running.

## Motivation

Most developers end up writing brittle ad-hoc checks in their shell configs or
tools:

- _"If I'm in VS Code, set `EDITOR=code -w`"_
- _"If stdout is piped, disable color and paging"_
- _"If running in a coding agent, simplify my prompt"_

These heuristics get duplicated across dotfiles and codebases. **envsense**
centralizes and standardizes this detection.

## Installation

### Pre-built Binaries (Recommended)

Download the latest release for your platform from
[GitHub Releases](https://github.com/your-org/envsense/releases):

```bash
# Linux x64
curl -L https://github.com/your-org/envsense/releases/latest/download/envsense-v0.1.0-x86_64-unknown-linux-gnu -o envsense
chmod +x envsense

# macOS Intel
curl -L https://github.com/your-org/envsense/releases/latest/download/envsense-v0.1.0-x86_64-apple-darwin -o envsense
chmod +x envsense

# macOS Universal (Intel + Apple Silicon)
curl -L https://github.com/your-org/envsense/releases/latest/download/envsense-v0.1.0-universal-apple-darwin -o envsense
chmod +x envsense

# macOS Apple Silicon (if you prefer architecture-specific)
curl -L https://github.com/your-org/envsense/releases/latest/download/envsense-v0.1.0-aarch64-apple-darwin -o envsense
chmod +x envsense

# Windows x64
curl -L https://github.com/your-org/envsense/releases/latest/download/envsense-v0.1.0-x86_64-pc-windows-msvc.exe -o envsense.exe
```

### From Source

```bash
# Install from source (requires Rust)
cargo install --git https://github.com/your-org/envsense
```

## Quick Start (Shell)

```bash
# Detect and show if running from an agent
envsense check agent # => true
# Detect and show agent id, if present
envsense check agent.id # => cursor

# Simple check for any coding agent
envsense -q check agent && echo "Running inside a coding agent"

# Check if specifically running in Cursor
envsense -q check agent.id=cursor && echo "Cursor detected"

# Check if running in VS Code or VS Code Insiders
envsense -q check ide.id=vscode && echo "VS Code"
envsense -q check ide.id=vscode-insiders && echo "VS Code Insiders"

# Check if running in GitHub Actions
envsense -q check ci.id=github && echo "GitHub Actions"

# Check if terminal is interactive
envsense -q check terminal.interactive && echo "Interactive terminal"

# Check multiple conditions (any must match)
envsense check --any agent ide && echo "Running in agent or IDE"

# Check multiple conditions (all must match)
envsense check --all agent terminal.interactive && echo "Interactive agent session"

# Get detailed reasoning for checks
envsense check --explain ide.id=cursor

# List all available predicates
envsense check --list
```

## Exploring the Environment

You can also inspect the full structured output to see what envsense detects:

```bash
# Human-friendly summary
envsense info

# Print detected environment as JSON
envsense info --json

# Pipe-friendly text
envsense info --raw

# Disable color output
envsense info --no-color
```

Example JSON output:

```json
{
  "contexts": ["agent", "ide"],
  "traits": {
    "agent": {
      "id": "claude-code"
    },
    "ide": {
      "id": "cursor"
    },
    "terminal": {
      "interactive": true,
      "color_level": "truecolor",
      "stdin": {
        "tty": true,
        "piped": false
      },
      "stdout": {
        "tty": true,
        "piped": false
      },
      "stderr": {
        "tty": true,
        "piped": false
      },
      "supports_hyperlinks": true
    }
  },
  "evidence": [],
  "version": "0.3.0"
}
```

## Command Line Options

### Check Command Options

The `check` command evaluates predicates against the environment and exits with
status 0 on success, 1 on failure.

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

- `--fields <list>` - Comma-separated keys to include: `contexts`, `traits`,
  `facets`, `meta`

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

- **Contexts** — broad categories of environment (`agent`, `ide`, `ci`).
- **Traits** — identifiers, capabilities or properties (`terminal.interactive`,
  `terminal.supports_hyperlinks`, `terminal.color_level`).
- **Evidence** — why envsense believes something (env vars, TTY checks, etc.),
  with confidence scores.

### Ask: Which category is it?

- _"Running in VS Code"_ → **Context/Trait** (`ide`, `ide.id=vscode`).
- _"Running in GitHub Actions"_ → **Trait** (`ci.id=github`).
- _"Running in CI at all"_ → **Context** (`ci`).
- _"Can print hyperlinks"_ → **Trait** (`terminal.supports_hyperlinks=true`).
- _"stdout is not a TTY"_ → **Trait** (`terminal.interactive=false`).

### Snippet Examples

```bash
# Context: any CI
envsense check ci && echo "In CI"

# Trait: specifically GitHub Actions
envsense check ci.id=github && echo "In GitHub Actions"

# Trait: non-interactive
if ! envsense check terminal.interactive; then
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

# Traits (specific identifiers and capabilities)
envsense check agent.id=cursor
envsense check ide.id=vscode
envsense check ci.id=github
envsense check terminal.interactive
envsense check terminal.supports_colors

# Negation (prefix with !)
envsense check !agent
envsense check !terminal.interactive
envsense check !ide.id=vscode
```

## Language Bindings

- **Rust** (Primary):

  ```rust
  use envsense::detect_environment;

  let env = detect_environment();
  if env.contexts.contains("agent") {
      println!("Agent detected");
  }
  if env.traits.agent.id.as_deref() == Some("cursor") {
      println!("Cursor detected");
  }
  if !env.traits.terminal.interactive {
      println!("Non-interactive session");
  }
  ```

- **Node.js** (Planned):

  ```js
  import { detect } from "envsense";
  const ctx = await detect();
  if (ctx.contexts.includes("agent")) console.log("Agent detected");
  if (ctx.traits.agent.id === "cursor") console.log("Cursor detected");
  if (!ctx.traits.terminal.interactive) console.log("Non-interactive session");
  ```

## Detection Strategy

1. **Explicit signals** — documented env vars (e.g. `TERM_PROGRAM`,
   `INSIDE_EMACS`, `CI`).
2. **Process ancestry** — parent process names (optional, behind a flag).
3. **Heuristics** — last resort (file paths, working dir markers).

Precedence is: user override > explicit > channel > ancestry > heuristics.

## Terminal Features Detected

- **Interactivity**
  - Shell flags (interactive mode)
  - TTY checks for stdin/stdout/stderr
  - Pipe/redirect detection
- **Colors**
  - Honors `NO_COLOR`, `FORCE_COLOR`
  - Detects depth: none, basic, 256, truecolor
- **Hyperlinks (OSC 8)**
  - Known supporting terminals (iTerm2, kitty, WezTerm, VS Code, etc.)
  - Optional probe for fallback

## Migration from v0.2.0

If you're upgrading from envsense v0.2.0, the syntax has been simplified:

| Old Syntax                  | New Syntax                     | Notes               |
| --------------------------- | ------------------------------ | ------------------- |
| `facet:agent_id=cursor`     | `agent.id=cursor`              | Direct mapping      |
| `facet:ide_id=vscode`       | `ide.id=vscode`                | IDE context         |
| `facet:ci_id=github`        | `ci.id=github`                 | CI context          |
| `trait:is_interactive`      | `terminal.interactive`         | Boolean field       |
| `trait:supports_hyperlinks` | `terminal.supports_hyperlinks` | Terminal capability |

For a complete migration guide, see
[docs/migration-guide.md](docs/migration-guide.md).

## Project Status

- **Rust implementation complete** - Core library and CLI fully functional
- **Declarative detection system** - Environment detection using declarative
  patterns
- **Comprehensive CI support** - GitHub Actions, GitLab CI, CircleCI, and more
- **Extensible architecture** - Easy to add new detection patterns

### Roadmap

- [x] Implement baseline detectors (env vars, TTY)
- [x] CLI output in JSON + pretty modes
- [x] Rule engine for contexts/facets/traits
- [x] Declarative detection system
- [x] Comprehensive CI environment support
- [ ] Node binding via napi-rs
- [ ] Additional language bindings
- [ ] Advanced value extraction patterns

## License

MIT

# Branch Protection Test
