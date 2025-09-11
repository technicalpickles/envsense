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

### Via Aqua/Mise (Recommended)

The easiest way to install `envsense` is via [aqua](https://aquaproj.github.io/)
through [mise](https://mise.jdx.dev/):

```bash
# Install via mise (which uses aqua)
mise install aqua:technicalpickles/envsense

# Or install globally
mise use -g aqua:technicalpickles/envsense
```

This method automatically:

- Downloads the correct binary for your platform
- Verifies cryptographic signatures via cosign
- Handles installation and PATH management

### Pre-built Binaries

Download the latest release for your platform from
[GitHub Releases](https://github.com/technicalpickles/envsense/releases):

```bash
# Get the latest version dynamically
LATEST_VERSION=$(curl -s https://api.github.com/repos/technicalpickles/envsense/releases/latest | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

# Linux x64
curl -L "https://github.com/technicalpickles/envsense/releases/latest/download/envsense-${LATEST_VERSION}-x86_64-unknown-linux-gnu" -o envsense
chmod +x envsense

# macOS Universal (Intel + Apple Silicon)
curl -L "https://github.com/technicalpickles/envsense/releases/latest/download/envsense-${LATEST_VERSION}-universal-apple-darwin" -o envsense
chmod +x envsense
```

**Alternative: One-liner install scripts**

For convenience, here are one-liner commands that handle version detection
automatically:

```bash
# Linux x64
curl -s https://api.github.com/repos/technicalpickles/envsense/releases/latest \
  | grep "browser_download_url.*x86_64-unknown-linux-gnu\"" \
  | grep -v "\.sig\|\.sha256\|\.bundle" \
  | cut -d '"' -f 4 \
  | xargs curl -L -o envsense && chmod +x envsense

# macOS (Universal - works on both Intel and Apple Silicon)
curl -s https://api.github.com/repos/technicalpickles/envsense/releases/latest \
  | grep "browser_download_url.*universal-apple-darwin\"" \
  | grep -v "\.sig\|\.sha256\|\.bundle\|v0\." \
  | cut -d '"' -f 4 \
  | xargs curl -L -o envsense && chmod +x envsense
```

> **Note**: Currently, Windows builds are not available. For Windows users,
> consider using [WSL](https://docs.microsoft.com/en-us/windows/wsl/) and
> following the Linux installation instructions.

### From Source

```bash
# Install from source (requires Rust)
cargo install --git https://github.com/technicalpickles/envsense
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
- `--descriptions` - Show context descriptions in list mode (requires `--list`)

#### Validation

- `--lenient` - Use lenient mode (don't error on invalid fields)

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
envsense check --list --descriptions   # Shows contexts with descriptions

# Lenient mode (for experimental usage)
envsense check --lenient unknown.field # Won't error on invalid field paths
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

#### Display Options

- `--tree` - Use tree structure for nested display (hierarchical is default)
- `--compact` - Compact output without extra formatting

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

# Display options
envsense info --tree                   # Tree structure display
envsense info --compact                # Compact formatting
envsense info --tree --compact         # Tree structure with compact formatting
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

## Configuration

envsense supports optional configuration via a TOML file located at
`~/.config/envsense/config.toml` (or equivalent on your platform).

### Configuration Structure

```toml
[error_handling]
strict_mode = true         # Enable strict validation (default: true)
show_usage_on_error = true # Show usage help on errors (default: true)

[output_formatting]
context_descriptions = true # Show descriptions in --list (default: true)
nested_display = true       # Use hierarchical output (default: true)
rainbow_colors = true       # Enable rainbow colors for special values (default: true)

[validation]
validate_predicates = true           # Validate predicate syntax (default: true)
allowed_characters = "a-zA-Z0-9_.=-" # Valid characters in predicates
```

### Configuration Loading

- Configuration is loaded automatically from the standard config directory
- If no config file exists, sensible defaults are used
- Partial configuration files are supported (missing sections use defaults)
- Configuration errors are silently ignored, falling back to defaults

### Creating a Configuration File

```bash
# Create the config directory
mkdir -p ~/.config/envsense

# Create a basic configuration file
cat > ~/.config/envsense/config.toml << EOF
[error_handling]
strict_mode = true

[output_formatting]
rainbow_colors = false

[validation]
validate_predicates = true
EOF
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

## Troubleshooting

### Aqua/Mise Installation Issues

If you encounter issues installing via aqua/mise, try these solutions:

**Permission denied or signature verification failed:**

```bash
# Ensure aqua policies allow the installation
mise exec aqua -- aqua policy allow 'technicalpickles/envsense'
```

**Binary not found after installation:**

```bash
# Check if binary is in PATH
mise which envsense

# Reload your shell
exec $SHELL

# Or explicitly activate mise in your shell profile
eval "$(mise activate)"
```

**Version-specific installation:**

```bash
# Install a specific version
mise install aqua:technicalpickles/envsense@0.3.4

# List available versions
mise ls-remote aqua:technicalpickles/envsense
```

**Cosign signature verification issues:** If you're in an environment that
doesn't support cosign verification:

- Consider using the direct binary download method instead
- Check if your environment has the necessary cosign dependencies
- Consult the [aqua documentation](https://aquaproj.github.io/) for signature
  verification troubleshooting

**GitHub API rate limiting:** If you see "GitHub rate limit exceeded" errors
with `mise install aqua:technicalpickles/envsense`:

```bash
# Wait for rate limit to reset (shown in the error message)
# Or authenticate with GitHub to get higher rate limits
gh auth login
# Then retry the installation
mise install aqua:technicalpickles/envsense
```

### General Issues

**Binary works but shows unexpected results:**

```bash
# Get detailed detection information
envsense info --json | jq '.'

# Enable debug logging to see detection process
ENVSENSE_LOG=debug envsense info
```

**False positives/negatives in environment detection:**

- Check your environment variables: `printenv | grep -E "(TERM|EDITOR|CI|IDE)"`
- Review process tree: `ps auxf` or `pstree`
- Consider using explicit overrides via environment variables

For more help, please
[open an issue](https://github.com/technicalpickles/envsense/issues) with:

- Your operating system and version
- Installation method used
- Full error messages
- Output of `envsense info --json`

## License

MIT
