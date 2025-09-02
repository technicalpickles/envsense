# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Development Commands

### Building and Running

- `cargo build` - Build the project
- `cargo run` - Run the CLI
- `cargo run -- info` - Show environment detection output
- `cargo run -- check <predicate>` - Test environment predicates

### Testing

- `cargo test` - Run all tests
- `cargo test --test cli` - Run CLI integration tests
- `cargo test --test info_snapshots` - Run snapshot tests

### CLI Usage Examples

- `cargo run -- info --json` - JSON output of detected environment
- `cargo run -- check agent` - Check if running in coding agent
- `cargo run -- check ide.id=cursor` - Check for Cursor specifically
- `cargo run -- check terminal.interactive` - Check if interactive terminal

### Snapshot Testing

- Tests use `insta` for snapshot testing of environment detection
- Snapshots are in `tests/snapshots/` directory
- Use `cargo insta review` to review snapshot changes
- Use `cargo insta accept` to accept all snapshot changes

## Architecture

### Core Components

**Detection Engine** (`src/schema.rs`):

- `EnvSense` - Main detection struct with `detect()` method
- Auto-detects on `Default::default()`
- Returns structured data: contexts, facets, traits, evidence

**CLI Interface** (`src/main.rs`):

- Two main commands: `info` (show detection) and `check` (evaluate predicates)
- Multiple output formats: human-readable, JSON, raw text
- Predicate system for querying detected environment

**Detection Modules**:

- `agent.rs` - Coding agent detection (Cursor, Claude Code, etc.)
- `ci.rs` - CI system detection (GitHub Actions, GitLab CI, etc.)
- `schema.rs` - IDE detection (VS Code, Cursor variants)
- `traits/terminal.rs` - Terminal capabilities (colors, hyperlinks, TTY)

### Key Data Structures

**Contexts** - Broad environment categories:

- `agent`, `ide`, `ci`, `container`, `remote`

**Facets** - Specific identifiers:

- `agent_id`, `ide_id`, `ci_id` with string values
- Special `ci` facet with vendor/name/pr/branch details

**Traits** - Specific identifiers and capabilities:

- `agent.id`, `ide.id`, `ci.id` with string values
- `terminal.interactive`, `terminal.supports_hyperlinks`
- TTY detection for stdin/stdout/stderr

**Evidence** - Detection reasoning with confidence scores

### Schema Stability

The JSON output schema is stable (version 0.3.0). Field names and enum values in
`CONTRACT.md` must not break without version bump. All consumer-visible keys use
`snake_case`.

### Testing Strategy

Uses comprehensive snapshot testing across different environments:

- Various IDEs (VS Code, Cursor)
- CI systems (GitHub Actions, GitLab CI)
- Terminal types (TTY, piped, tmux)
- Operating systems (macOS, Linux)

Test data includes both `.env` files (environment setup) and `.json` files
(expected output) for each scenario.
