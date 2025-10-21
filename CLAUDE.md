# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Project Overview

**envsense** is a cross-language environment detection library and CLI that
detects runtime environments (IDEs, coding agents, CI systems, terminals) and
exposes structured information to help tools adapt their behavior. The JSON
output schema is stable (v0.3.0) and must maintain backward compatibility as
defined in [CONTRACT.md](CONTRACT.md).

## Development Commands

### Building and Running

- `cargo build` - Build the project
- `cargo run` - Run the CLI
- `cargo run -- info` - Show environment detection output
- `cargo run -- info --json` - JSON output of detected environment
- `cargo run -- check <predicate>` - Test environment predicates

### Testing

- `cargo test` - Run all tests
- `cargo test --test <name>` - Run specific test file (e.g., `cli`,
  `info_snapshots`)
- `cargo test <pattern>` - Run tests matching pattern

### Snapshot Testing

- Tests use `insta` for snapshot testing of environment detection
- Snapshots are in `tests/snapshots/` directory
- `cargo insta review` - Review snapshot changes interactively
- `cargo insta accept` - Accept all snapshot changes
- `cargo insta test` - Run tests and review snapshots in one command

### CLI Usage Examples

- `cargo run -- info --json` - JSON output of detected environment
- `cargo run -- check agent` - Check if running in coding agent
- `cargo run -- check agent.id=cursor` - Check for Cursor specifically
- `cargo run -- check ide.id=vscode` - Check for VS Code
- `cargo run -- check terminal.interactive` - Check if interactive terminal
- `cargo run -- check --explain agent` - Show reasoning for detection

## Architecture

### Core Components

**Detection Engine** ([src/engine.rs](src/engine.rs),
[src/schema/](src/schema/)):

- `EnvSense` - Main detection struct with `detect()` method
- Auto-detects on `Default::default()`
- Returns structured data: contexts, traits, evidence, version
- Located in `src/schema/main.rs` with supporting modules

**CLI Interface** ([src/main.rs](src/main.rs)):

- Two main commands: `info` (show detection) and `check` (evaluate predicates)
- Multiple output formats: human-readable, JSON, raw text
- Predicate system for querying detected environment
- Configuration support via TOML files

**Detection System** ([src/detectors/](src/detectors/)):

- **Declarative Detection** - Pattern-based detection using TOML definitions
  - `agent_declarative.rs` - Coding agent detection (Cursor, Claude Code, etc.)
  - `ci_declarative.rs` - CI system detection (GitHub Actions, GitLab CI, etc.)
  - `ide_declarative.rs` - IDE detection (VS Code, Cursor variants, JetBrains,
    etc.)
  - `declarative.rs` - Core declarative detection engine
- **Terminal Detection** - TTY, color support, hyperlink capabilities
  - `terminal.rs` - Terminal feature detection
  - `tty.rs` - TTY stream detection (stdin/stdout/stderr)
- **Utilities** - Helper functions and environment mapping
  - `env_mapping.rs` - Environment variable value extraction
  - `utils.rs` - Common detection utilities

**Traits** ([src/traits/](src/traits/)):

- `agent.rs` - Agent-specific traits (agent ID)
- `ci.rs` - CI-specific traits (vendor, PR detection, branch)
- `ide.rs` - IDE-specific traits (IDE ID)
- `terminal.rs` - Terminal capabilities (interactive, colors, hyperlinks, TTY)
- `stream.rs` - Stream-specific traits (piped, TTY status)
- `nested.rs` - Support for nested trait structures

**Macros** ([envsense-macros/](envsense-macros/)):

- Custom derive macros for automatic trait field extraction
- Supports nested trait structures and field mapping

### Key Data Structures

**Contexts** - Broad environment categories:

- `agent`, `ide`, `ci`, `container`, `remote`

**Traits** - Nested structure containing specific identifiers and capabilities:

- `agent.id` - Agent identifier (e.g., "cursor", "claude-code")
- `ide.id` - IDE identifier (e.g., "vscode", "cursor")
- `ci.id` - CI system identifier (e.g., "github", "gitlab")
- `ci.vendor`, `ci.name`, `ci.pr`, `ci.branch` - CI metadata
- `terminal.interactive` - Boolean for TTY interactivity
- `terminal.color_level` - Color support level (none, ansi16, ansi256,
  truecolor)
- `terminal.supports_hyperlinks` - OSC 8 hyperlink support
- `terminal.stdin`, `terminal.stdout`, `terminal.stderr` - Stream properties
  (tty, piped)

**Evidence** - Detection reasoning with confidence scores:

- `signal` - Detection method (env, tty, proc, fs)
- `key` - Environment variable or indicator name
- `value` - Detected value
- `supports` - What contexts/traits this evidence supports
- `confidence` - Reliability score (0.0-1.0)

### Schema Stability

The JSON output schema is stable (version 0.3.0). All changes must maintain
backward compatibility as defined in [CONTRACT.md](CONTRACT.md):

- Field renames require `serde` aliases
- Field removals require version bump
- New fields and enum values can be added freely
- All consumer-visible keys use `snake_case`

### Detection Strategy

Declarative pattern-based detection system with precedence order:

1. **User override** - Explicit configuration or flags
2. **Explicit signals** - Well-documented environment variables
3. **Channel** - Execution environment context (SSH, containers)
4. **Ancestry** - Process parent/child relationships (optional, behind flag)
5. **Heuristics** - File system and working directory analysis (last resort)

### Testing Strategy

**Snapshot Testing** (`tests/info_snapshots.rs`):

- Uses `insta` crate for comprehensive snapshot testing
- Test scenarios in `tests/scenarios/` directory
- Each scenario has `.env` (setup) and `.json` (expected output)
- Covers: IDEs, CI systems, terminal types, operating systems

**Integration Tests** (`tests/` directory):

- CLI behavior tests (`cli*.rs`)
- Declarative detection tests (`declarative*.rs`)
- Macro functionality tests (`macro*.rs`)
- Terminal and configuration tests

**Test Utilities** (`src/detectors/test_utils.rs`):

- Mock environment setup
- Test helper functions

## Workspace Structure

This is a Cargo workspace with multiple crates:

- **`envsense`** (root) - Main library and CLI
- **`envsense-macros`** - Macro crate for custom derives
- **`envsense-macros/envsense-macros-impl`** - Procedural macro implementation

Build all workspace members with `cargo build` from the root directory.

## Important Notes

### Schema Compatibility

When modifying detection logic or output structures:

- Review [CONTRACT.md](CONTRACT.md) for schema stability requirements
- Existing field names and enum values cannot change without version bump
- Use `#[serde(rename = "...")]` or `#[serde(alias = "...")]` if refactoring
- All JSON output must use `snake_case` for field names

### Adding New Detections

To add support for a new IDE, CI system, or agent:

1. Add detection patterns to appropriate declarative module:
   - `src/detectors/agent_declarative.rs` for agents
   - `src/detectors/ci_declarative.rs` for CI systems
   - `src/detectors/ide_declarative.rs` for IDEs
2. Add test scenarios in `tests/scenarios/`
3. Run `cargo insta test` to generate and review snapshots
4. Update documentation if adding new trait fields

### Declarative Detection Pattern

The project uses a declarative pattern-matching system for environment
detection. Instead of imperative detection logic, environment variables and
patterns are defined in declarative structures, making it easier to add new
detection cases without complex code changes.
