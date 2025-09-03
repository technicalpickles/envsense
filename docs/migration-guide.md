# Migration Guide: v0.2.0 to v0.3.0

## Overview

envsense v0.3.0 introduces significant improvements to the CLI interface and
schema structure. This version removes the legacy `facet:` and `trait:` syntax
in favor of a more intuitive dot notation system. The migration is designed to
be straightforward with comprehensive tooling support.

## Why Migration is Needed

The legacy syntax had several limitations:

- **Verbose**: `facet:agent_id=cursor` vs `agent.id=cursor`
- **Inconsistent**: Mixed `facet:` and `trait:` prefixes
- **Less intuitive**: Harder to understand the relationship between fields
- **Maintenance burden**: Complex parsing logic for legacy syntax

The new dot notation provides:

- **Cleaner syntax**: `agent.id=cursor` is more readable
- **Consistent structure**: All fields use the same pattern
- **Better discoverability**: Field relationships are clearer
- **Improved performance**: Simplified parsing and evaluation

## Breaking Changes

### Schema Changes

- **Schema version**: Updated from `0.2.0` to `0.3.0`
- **Contexts structure**: Now returned as array instead of object
- **Field paths**: All fields now use dot notation (`agent.id`,
  `terminal.interactive`)

### CLI Syntax Changes

- **Removed**: `facet:` prefix for identifier fields
- **Removed**: `trait:` prefix for capability fields
- **Added**: Direct field access with dot notation
- **Updated**: All examples and help text

### API Changes

- **Rust**: `env.contexts.agent` → `env.contexts.contains("agent")`
- **Rust**: `env.facets.ide_id` → `env.traits.ide.id`
- **Rust**: `env.traits.is_interactive` → `env.traits.terminal.interactive`

## Migration Steps

### Step 1: Update CLI Commands

Replace all legacy syntax in your scripts and commands:

```bash
# OLD (v0.2.0)
envsense check facet:agent_id=cursor
envsense check trait:is_interactive
envsense check facet:ci_id=github

# NEW (v0.3.0)
envsense check agent.id=cursor
envsense check terminal.interactive
envsense check ci.id=github
```

### Step 2: Update Configuration Files

If you have JSON configuration files, use the migration tools:

```bash
# Migrate a JSON file
envsense migrate --json config.json

# Validate migration
envsense migrate --validate config.json
```

### Step 3: Update Code

Update your Rust code to use the new API:

```rust
// OLD (v0.2.0)
let env = detect_environment();
if env.contexts.agent {
    println!("Agent detected");
}
if env.facets.ide_id.as_deref() == Some("cursor") {
    println!("Cursor detected");
}

// NEW (v0.3.0)
let env = detect_environment();
if env.contexts.contains("agent") {
    println!("Agent detected");
}
if env.traits.ide.id.as_deref() == Some("cursor") {
    println!("Cursor detected");
}
```

### Step 4: Test and Validate

Verify that your migrated code works correctly:

```bash
# Test individual predicates
envsense check agent.id=cursor
envsense check terminal.interactive

# Test JSON output
envsense info --json | jq '.version'
# Should show "0.3.0"
```

## Complete Syntax Mapping

### Agent Detection

| Legacy Syntax            | New Syntax         | Description    |
| ------------------------ | ------------------ | -------------- |
| `facet:agent_id=cursor`  | `agent.id=cursor`  | Cursor agent   |
| `facet:agent_id=claude`  | `agent.id=claude`  | Claude agent   |
| `facet:agent_id=github`  | `agent.id=github`  | GitHub Copilot |
| `facet:agent_id=copilot` | `agent.id=copilot` | GitHub Copilot |

### IDE Detection

| Legacy Syntax                  | New Syntax               | Description      |
| ------------------------------ | ------------------------ | ---------------- |
| `facet:ide_id=vscode`          | `ide.id=vscode`          | VS Code          |
| `facet:ide_id=vscode-insiders` | `ide.id=vscode-insiders` | VS Code Insiders |
| `facet:ide_id=cursor`          | `ide.id=cursor`          | Cursor           |
| `facet:ide_id=neovim`          | `ide.id=neovim`          | Neovim           |

### CI Detection

| Legacy Syntax          | New Syntax       | Description     |
| ---------------------- | ---------------- | --------------- |
| `facet:ci_id=github`   | `ci.id=github`   | GitHub Actions  |
| `facet:ci_id=gitlab`   | `ci.id=gitlab`   | GitLab CI       |
| `facet:ci_id=circle`   | `ci.id=circle`   | CircleCI        |
| `facet:ci_id=jenkins`  | `ci.id=jenkins`  | Jenkins         |
| `facet:ci_branch=main` | `ci.branch=main` | CI branch name  |
| `facet:ci_pr=123`      | `ci.pr=123`      | CI pull request |

### Terminal Traits

| Legacy Syntax               | New Syntax                     | Description          |
| --------------------------- | ------------------------------ | -------------------- |
| `trait:is_interactive`      | `terminal.interactive`         | Interactive terminal |
| `trait:is_tty_stdin`        | `terminal.stdin.tty`           | stdin is TTY         |
| `trait:is_tty_stdout`       | `terminal.stdout.tty`          | stdout is TTY        |
| `trait:is_tty_stderr`       | `terminal.stderr.tty`          | stderr is TTY        |
| `trait:is_piped_stdin`      | `terminal.stdin.piped`         | stdin is piped       |
| `trait:is_piped_stdout`     | `terminal.stdout.piped`        | stdout is piped      |
| `trait:supports_hyperlinks` | `terminal.supports_hyperlinks` | Hyperlink support    |
| `trait:supports_colors`     | `terminal.supports_colors`     | Color support        |

### Container Detection

| Legacy Syntax               | New Syntax            | Description      |
| --------------------------- | --------------------- | ---------------- |
| `facet:container_id=docker` | `container.id=docker` | Docker container |
| `facet:container_id=podman` | `container.id=podman` | Podman container |

## CLI Migration Tools

### Migration Command

The `envsense migrate` command provides comprehensive migration support:

```bash
# Show migration guide
envsense migrate --guide

# Migrate a single predicate
envsense migrate --predicate "facet:agent_id=cursor"
# Output: agent.id=cursor

# Migrate a JSON file
envsense migrate --json legacy-config.json

# Validate migration
envsense migrate --validate legacy-config.json
```

### Migration Examples

```bash
# Interactive migration
envsense migrate --predicate "facet:ide_id=vscode"
# Output: ide.id=vscode

# Batch migration
echo "facet:agent_id=cursor" | envsense migrate --predicate -
# Output: agent.id=cursor

# JSON migration
envsense info --json > legacy.json
envsense migrate --json legacy.json > new.json
```

## Common Issues and Solutions

### Issue: "Unknown predicate" Error

**Problem**: After migration, you get "Unknown predicate" errors.

**Solution**: Verify the field path exists in the new schema:

```bash
# List all available predicates
envsense check --list

# Check if a specific field exists
envsense check agent.id
```

### Issue: Different Results After Migration

**Problem**: Migrated predicates produce different results.

**Solution**: Use the validation tools to check for discrepancies:

```bash
# Validate migration correctness
envsense migrate --validate "facet:agent_id=cursor"

# Compare outputs
envsense check "facet:agent_id=cursor"  # Legacy (if still supported)
envsense check "agent.id=cursor"         # New syntax
```

### Issue: JSON Schema Mismatch

**Problem**: JSON output structure has changed.

**Solution**: Update your JSON parsing code:

```json
// OLD (v0.2.0)
{
  "contexts": {
    "agent": true,
    "ide": true
  },
  "facets": {
    "agent_id": "cursor"
  }
}

// NEW (v0.3.0)
{
  "contexts": ["agent", "ide"],
  "traits": {
    "agent": {
      "id": "cursor"
    }
  }
}
```

### Issue: Script Failures

**Problem**: Shell scripts fail after migration.

**Solution**: Update all script references:

```bash
# OLD
if envsense check -q facet:agent_id=cursor; then
    echo "Cursor detected"
fi

# NEW
if envsense check -q agent.id=cursor; then
    echo "Cursor detected"
fi
```

## Rollback Plan

If you encounter critical issues during migration:

1. **Downgrade**: Install v0.2.0

   ```bash
   cargo install envsense --version 0.2.0
   ```

2. **Restore**: Use your backup configuration files

3. **Report**: Open an issue with details about the problem

4. **Gradual Migration**: Migrate one component at a time

## Testing Your Migration

### Automated Testing

Create test scripts to verify migration correctness:

```bash
#!/usr/bin/env bash
# test-migration.sh

# Test all migrated predicates
predicates=(
    "agent.id=cursor"
    "ide.id=vscode"
    "terminal.interactive"
    "ci.id=github"
)

for pred in "${predicates[@]}"; do
    if ! envsense check -q "$pred"; then
        echo "FAIL: $pred"
        exit 1
    fi
    echo "PASS: $pred"
done

echo "All migration tests passed!"
```

### Manual Verification

Test key functionality manually:

```bash
# Basic functionality
envsense check agent
envsense check ide
envsense check ci

# Specific checks
envsense check agent.id=cursor
envsense check terminal.interactive

# JSON output
envsense info --json | jq '.version'
```

## Performance Impact

The new syntax provides several performance improvements:

- **Faster parsing**: Simplified predicate parsing
- **Reduced memory**: Cleaner data structures
- **Better caching**: Improved field resolution
- **Optimized evaluation**: Streamlined trait checking

## Support and Resources

### Documentation

- **Current Guide**: This migration guide
- **API Reference**: `cargo doc --open`
- **CLI Help**: `envsense --help`

### Community

- **GitHub Issues**: Report bugs and request features
- **Discussions**: Ask questions and share experiences
- **Contributing**: Help improve envsense

### Getting Help

If you encounter issues during migration:

1. **Check this guide** for common solutions
2. **Search existing issues** for similar problems
3. **Open a new issue** with detailed information
4. **Include migration context** in your report

## Conclusion

The migration to envsense v0.3.0 represents a significant improvement in
usability and maintainability. While the changes require updates to existing
code and configurations, the benefits include:

- **Cleaner syntax** that's easier to read and write
- **Better performance** through simplified parsing
- **Improved maintainability** with consistent patterns
- **Enhanced discoverability** of available fields

The migration tools and comprehensive documentation ensure a smooth transition.
Take advantage of the validation features to verify correctness, and don't
hesitate to seek help if you encounter issues.

Welcome to envsense v0.3.0!
