# Additional CLI Improvements

Beyond the core CLI streamlining changes, several enhancements can be made to
improve user experience and error handling.

## Enhanced Error Handling

### `envsense check` (No Arguments)

**Current Behavior**: Returns "no predicates" error **Proposed Improvement**:
Provide more helpful usage information

```bash
# Current output
$ envsense check
Error: no predicates

# Proposed output
$ envsense check
Error: no predicates specified

Usage: envsense check <predicate> [<predicate>...]

Examples:
  envsense check agent                    # Check if running in an agent
  envsense check ide.cursor              # Check if Cursor IDE is active
  envsense check ci.github               # Check if in GitHub CI
  envsense check agent.id=cursor         # Check specific agent ID
  envsense check --list                  # List all available predicates

For more information, see: envsense check --help
```

**Implementation Notes**:

- Update error message to be more descriptive
- Include usage examples in error output
- Suggest `--list` flag for discovery
- Reference help command for detailed information

### Invalid Predicate Syntax

**Current Behavior**: `envsense check ide.nonexistent` returns `false`
**Proposed Improvement**: Consider whether to raise errors for invalid fields

**Options**:

1. **Strict Mode** (Recommended): Raise error for invalid field paths

   ```bash
   $ envsense check ide.nonexistent
   Error: invalid field path 'ide.nonexistent'

   Available fields for 'ide':
     - id: The IDE identifier
     - name: The IDE name
     - version: The IDE version

   Use 'envsense info ide' to see all available fields
   ```

2. **Lenient Mode**: Keep current behavior (return `false`)
   ```bash
   $ envsense check ide.nonexistent
   false
   ```

**Recommendation**: Implement strict mode with `--lenient` flag for backward
compatibility.

### Special Character Validation

**Current Behavior**: `envsense check ?asdfasdf` may have undefined behavior
**Proposed Improvement**: Validate predicate syntax and provide clear errors

```bash
# Invalid characters
$ envsense check ?asdfasdf
Error: invalid predicate syntax '?asdfasdf'

Valid predicate syntax:
  - Simple context: agent, ide, ci
  - Field access: agent.id, ide.name
  - Comparison: agent.id=cursor, ci.name=github
  - No special characters except dots (.) and equals (=)

Examples of valid predicates:
  agent
  ide.cursor
  ci.github
  agent.id=cursor
```

**Implementation Notes**:

- Define allowed character set for predicates
- Validate syntax before parsing
- Provide clear examples of valid syntax
- Consider regex pattern: `^[a-zA-Z][a-zA-Z0-9_.=]*$`

### Logical Flag Validation

**Current Behavior**: Some flag combinations don't make logical sense but are
allowed **Proposed Improvement**: Validate flag combinations and provide clear
error messages

#### Invalid Flag Combinations

```bash
# --list with --any/--all (logical conflict)
$ envsense check --list --any
Error: invalid flag combination: --list cannot be used with --any or --all

The --list flag shows available predicates, while --any/--all control evaluation logic.
These flags serve different purposes and cannot be combined.

Usage examples:
  envsense check --list                    # List available predicates
  envsense check --any agent ide          # Check if ANY predicate is true
  envsense check --all agent ide          # Check if ALL predicates are true
```

```bash
# --list with predicates (redundant)
$ envsense check agent --list
Error: invalid flag combination: --list cannot be used with predicates

The --list flag shows all available predicates, so providing specific predicates is redundant.

Usage examples:
  envsense check --list                    # List all available predicates
  envsense check agent                    # Check specific predicate
  envsense check agent ide                # Check multiple predicates
```

```bash
# --list with --quiet (contradictory)
$ envsense check --list --quiet
Error: invalid flag combination: --list cannot be used with --quiet

The --list flag is designed to show information, while --quiet suppresses output.
These flags have contradictory purposes and cannot be combined.

Usage examples:
  envsense check --list                    # Show available predicates
  envsense check agent --quiet            # Check predicate quietly
```

#### Valid Flag Combinations

```bash
# Logical evaluation flags
envsense check --any agent ide            # ANY predicate true
envsense check --all agent ide            # ALL predicates true
envsense check agent ide                  # Default: ALL predicates true

# Output control flags
envsense check agent --quiet              # Suppress output
envsense check agent --json               # JSON output
envsense check agent --pretty             # Pretty-printed output

# Discovery and help
envsense check --list                     # List available predicates
envsense check --help                     # Show help
```

**Implementation Notes**:

- Validate flag combinations before processing
- Provide clear error messages explaining why combinations are invalid
- Suggest alternative usage patterns
- Consider grouping flags by purpose (evaluation, output, discovery)
- Maintain backward compatibility for valid combinations

### Flag Purpose Groups

**Evaluation Flags** (mutually exclusive):

- `--any`: Return true if ANY predicate matches
- `--all`: Return true if ALL predicates match (default)
- `--list`: Show available predicates (cannot combine with evaluation)

**Output Control Flags**:

- `--quiet`: Suppress output (cannot combine with `--list`)
- `--json`: JSON output format
- `--pretty`: Pretty-printed output

**Discovery Flags**:

- `--list`: Show available predicates
- `--help`: Show help information

**Implementation Priority**:

1. **High**: Validate `--list` combinations (most obvious conflicts)
2. **Medium**: Validate evaluation flag conflicts
3. **Low**: Validate output control conflicts

## Enhanced Output Formatting

### Context Listing in `info` Command

**Current Behavior**: Contexts may be displayed in various formats **Proposed
Improvement**: Consistent, readable listing format

```bash
# Current output (varies)
$ envsense info
Contexts: agent, ide, ci, runtime, os, ...

# Proposed output
$ envsense info
Available contexts:
- agent: Agent environment detection
- ide: Integrated development environment
- ci: Continuous integration environment
- runtime: Runtime environment details
- os: Operating system information
- terminal: Terminal characteristics
- shell: Shell environment details
```

**Implementation Notes**:

- One context per line with leading dash
- Include brief description for each context
- Consistent formatting across all output modes
- Consider `--no-descriptions` flag for compact output

### Nested Value Display

**Current Behavior**: Nested values like `stdin/stdout/stderr` may appear as
JSON **Proposed Improvement**: Hierarchical, readable display

```bash
# Current output
$ envsense info terminal
{
  "stdin": {
    "tty": true,
    "piped": false
  },
  "stdout": {
    "tty": true,
    "piped": false
  }
}

# Proposed output
$ envsense info terminal
terminal:
  stdin:
    tty: true
    piped: false
  stdout:
    tty: true
    piped: false
  stderr:
    tty: true
    piped: false
```

**Implementation Notes**:

- Use consistent indentation for nesting
- Remove JSON formatting for human-readable output
- Maintain JSON output with `--json` flag
- Consider `--tree` flag for explicit tree structure

### Rainbow `colorlevel` Display

**Current Behavior**: `colorlevel` value shown as plain text **Proposed
Improvement**: Colorful, rainbow display for `truecolor` values

```bash
# Current output
$ envsense info terminal.color_level
truecolor

# Proposed output
$ envsense info terminal.color_level
truecolor

# With rainbow effect (when supported)
$ envsense info terminal.color_level --rainbow
T R U E C O L O R
```

**Implementation Notes**:

- Detect terminal color support
- Use different color per character for rainbow effect
- Fall back to plain text for non-color terminals
- Consider `--no-rainbow` flag to disable
- Implement color cycling for dynamic effect
- Focus on `ColorLevel::Truecolor` values specifically

## Implementation Priority

### Phase 1: Error Handling (High Priority)

- [ ] Improve `check` command usage errors
- [ ] Implement predicate syntax validation
- [ ] Add special character filtering
- [ ] Consider strict vs. lenient mode for invalid fields
- [ ] Validate logical flag combinations (--list conflicts)

### Phase 2: Output Formatting (Medium Priority)

- [ ] Standardize context listing format
- [ ] Implement nested value display
- [ ] Add rainbow `colorlevel` support for `truecolor`
- [ ] Maintain backward compatibility
- [ ] Validate evaluation flag conflicts (--any/--all)

### Phase 3: Polish and Testing (Lower Priority)

- [ ] Add configuration options for behavior
- [ ] Comprehensive error message testing
- [ ] User experience validation
- [ ] Performance impact assessment
- [ ] Validate output control flag conflicts

## Configuration Options

Consider adding these configuration options:

```toml
[cli]
# Error handling
strict_mode = true         # Raise errors for invalid fields
show_usage_on_error = true # Include usage examples in errors

# Output formatting
context_descriptions = true # Show context descriptions
nested_display = true       # Use hierarchical display
rainbow_colors = true       # Enable rainbow colorlevel display

# Validation
validate_predicates = true          # Validate predicate syntax
allowed_characters = "a-zA-Z0-9_.="
```

## Testing Strategy

### Error Handling Tests

- [ ] Test all error scenarios with appropriate messages
- [ ] Verify usage examples are accurate
- [ ] Test invalid predicate syntax handling
- [ ] Validate special character filtering

### Output Formatting Tests

- [ ] Snapshot tests for new formatting
- [ ] Terminal color support detection
- [ ] Nested value display consistency
- [ ] Context listing format validation

### Integration Tests

- [ ] End-to-end CLI behavior
- [ ] Error message consistency
- [ ] Output format consistency
- [ ] Performance impact measurement

## Technical Considerations

### Color Support Detection

- Use existing `ColorLevel` enum values
- Check `terminal.color_level` field
- Only apply rainbow effect to `ColorLevel::Truecolor`
- Fall back gracefully for other color levels

### Field Validation

- Leverage existing field registry
- Validate against known field paths
- Provide helpful suggestions for similar fields
- Consider fuzzy matching for typos

### Performance Impact

- Minimal overhead for validation
- Lazy loading of field descriptions
- Cached color support detection
- Efficient predicate parsing

## Migration Considerations

### Backward Compatibility

- All existing commands continue to work
- New flags are optional
- Configuration changes are additive
- Error messages can be disabled

### User Experience

- Gradual introduction of new features
- Clear documentation of changes
- Migration guides for power users
- Feedback collection mechanisms
