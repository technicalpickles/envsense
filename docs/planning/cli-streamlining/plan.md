# CLI Streamlining Plan

## Overview

This document outlines the planned changes to streamline the envsense CLI
interface, making it more intuitive and user-friendly while maintaining
functionality for the most common use cases.

## Motivation

The current CLI interface has several usability issues:

1. **Verbose predicate syntax** - `facet:agent_id=cursor` is less intuitive than
   `agent.id=cursor`
2. **Complex JSON structure** - Flat `facets` and `traits` objects are harder to
   navigate
3. **Over-documentation** - Development sections clutter the user-facing README
4. **Scattered terminal information** - Terminal capabilities are spread across
   multiple flat fields

## Proposed Changes

### 1. Simplified Predicate Syntax

**Current**: `facet:agent_id=cursor`, `trait:is_interactive` **Proposed**:
`agent.id=cursor`, `terminal.interactive`

**Benefits**:

- More intuitive dot notation
- Easier to remember and type
- Consistent with common programming conventions
- Reduces cognitive load

**Examples**:

```bash
# Current
envsense check facet:agent_id=cursor
envsense check trait:is_interactive

# Proposed
envsense check agent.id=cursor
envsense check terminal.interactive
```

### 2. Restructured JSON Schema

**Current Structure**:

```json
{
  "contexts": ["agent", "ide"],
  "facets": {
    "agent_id": "cursor",
    "ide_id": "cursor"
  },
  "traits": {
    "is_interactive": true,
    "is_tty_stdin": true,
    "is_tty_stdout": true,
    "is_tty_stderr": true,
    "is_piped_stdin": false,
    "is_piped_stdout": false,
    "color_level": "truecolor",
    "supports_hyperlinks": true
  }
}
```

**Proposed Structure**:

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
  }
}
```

**Benefits**:

- Logical grouping of related properties
- Easier to navigate and understand
- Better reflects the hierarchical nature of environment detection
- More intuitive for programmatic access

### 3. Enhanced Terminal Detection

**Current**: Flat boolean fields for TTY and piped status **Proposed**:
Structured terminal object with detailed state information

**Benefits**:

- More granular control over terminal capabilities
- Better organized terminal information
- Easier to extend with additional terminal features
- Clearer separation of concerns

### 4. Streamlined Documentation

**Removed Sections**:

- Development workflow documentation
- CI detection examples
- Container/remote context documentation
- Verbose field filtering examples

**Benefits**:

- Focus on user-facing features
- Reduced cognitive load
- Clearer value proposition
- Better first-time user experience

## Implementation Plan

### Phase 1: Schema Changes

1. Update `src/schema.rs` to reflect new nested structure
2. Modify trait definitions to use new organization
3. Update JSON serialization/deserialization
4. Bump schema version

### Phase 2: CLI Parser Updates

1. Implement new predicate syntax parser
2. Add support for dot notation
3. Maintain backward compatibility during transition
4. Update help text and examples

### Phase 3: Detection Logic

1. Update detectors to populate new schema structure
2. Modify terminal detection to use nested format
3. Update trait mapping logic
4. Ensure all existing functionality is preserved

### Phase 4: Documentation

1. Update README.md with new examples
2. Revise CLI help text
3. Update integration tests
4. Create migration guide for existing users

## Migration Strategy

### Backward Compatibility

- Maintain support for old predicate syntax during transition period
- Provide deprecation warnings for old syntax
- Document migration path for users

### Breaking Changes

- New JSON schema structure (requires schema version bump)
- New predicate syntax (old syntax will be deprecated)
- Removed contexts (container, remote)

### User Impact

- **Scripts**: Will need to update predicate syntax
- **JSON consumers**: Will need to adapt to new structure
- **Library users**: May need to update trait access patterns

## Risk Assessment

### High Risk

- **Breaking changes**: Users with existing scripts will need to migrate
- **Schema changes**: JSON consumers will need updates
- **Feature removal**: Container/remote contexts may be needed by some users

### Medium Risk

- **Parser complexity**: New predicate syntax may be harder to implement
- **Testing burden**: Need comprehensive test updates
- **Documentation effort**: Significant documentation updates required

### Low Risk

- **Performance impact**: Minimal performance changes expected
- **Memory usage**: Slightly more structured data but negligible impact

## Success Metrics

### User Experience

- Reduced time to first successful use
- Fewer support questions about predicate syntax
- Increased adoption in scripts and tools

### Technical

- Cleaner codebase with better organization
- More maintainable schema structure
- Easier to extend with new features

### Community

- Positive feedback on new interface
- Successful migration of existing users
- Increased contribution activity

## Alternatives Considered

### 1. Gradual Migration

- Keep both syntaxes supported indefinitely
- **Pros**: No breaking changes
- **Cons**: Increased complexity, maintenance burden

### 2. Minimal Changes

- Only update JSON structure, keep predicate syntax
- **Pros**: Less disruptive
- **Cons**: Misses opportunity for major UX improvement

### 3. Complete Redesign

- More radical interface changes
- **Pros**: Could address more fundamental issues
- **Cons**: Too disruptive, higher risk

## Conclusion

The proposed CLI streamlining represents a significant improvement to the user
experience while maintaining the core functionality that makes envsense
valuable. The dot notation syntax is more intuitive, the nested JSON structure
is better organized, and the focus on user-facing documentation improves the
onboarding experience.

The breaking changes are justified by the substantial UX improvements, and the
migration strategy provides a clear path for existing users to adapt to the new
interface.

## Next Steps

1. **Review and approval** of this plan
2. **Implementation** following the phased approach
3. **Testing** with existing users and use cases
4. **Documentation** updates and migration guide creation
5. **Release** with clear migration instructions
