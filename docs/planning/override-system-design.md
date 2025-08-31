# Override System Design

## Overview

This document details the design and implementation of a comprehensive override
system for all declarative detectors in envsense. The override system allows
users to control detection behavior through environment variables for testing,
debugging, and custom environments.

## Implementation Status

**Overall Progress**: 100% Complete ✅

### ✅ Completed Implementation

The comprehensive override system has been successfully implemented and is now
available for all detector types:

- ✅ **Agent Detection**: Existing overrides maintained (`ENVSENSE_AGENT`,
  `ENVSENSE_ASSUME_HUMAN`)
- ✅ **IDE Detection**: New overrides implemented (`ENVSENSE_IDE`,
  `ENVSENSE_ASSUME_TERMINAL`)
- ✅ **CI Detection**: New overrides implemented (`ENVSENSE_CI`,
  `ENVSENSE_ASSUME_LOCAL`)
- ✅ **Consistent Pattern**: All detectors follow the same override schema
- ✅ **Backward Compatibility**: All existing agent overrides continue to work
- ✅ **Comprehensive Testing**: All override scenarios tested and validated

## Current State

### Existing Overrides (Agent Detector Only)

| **Variable**              | **Behavior**                 | **Use Case**           |
| ------------------------- | ---------------------------- | ---------------------- |
| `ENVSENSE_ASSUME_HUMAN=1` | Disables all agent detection | Testing, debugging     |
| `ENVSENSE_AGENT=none`     | Same as ASSUME_HUMAN         | Alternative syntax     |
| `ENVSENSE_AGENT=<value>`  | Forces specific agent ID     | Custom agents, testing |

### Limitations

1. **Inconsistent**: Only agent detector supports overrides
2. **Limited Scope**: No overrides for IDE or CI detection
3. **Testing Gaps**: Difficult to test edge cases in IDE/CI detection
4. **Debugging**: No way to disable specific detectors for troubleshooting

## Proposed Override System

### Design Principles

1. **Consistency**: Same override pattern across all detectors
2. **Intuitive**: Clear, predictable variable names
3. **Flexible**: Support both disable and force scenarios
4. **Backward Compatible**: Maintain existing agent overrides
5. **Testable**: Enable comprehensive testing scenarios

### Override Variable Schema

#### Pattern 1: Direct Override

```bash
ENVSENSE_{DETECTOR}=<value>
```

- `none` = Disable detection entirely
- `<custom-value>` = Force specific detection result

#### Pattern 2: Assume Override

```bash
ENVSENSE_ASSUME_{MODE}=1
```

- Disables related detection types
- More semantic than direct disable

### Complete Override Variables

#### Agent Detection (Existing - Maintained)

```bash
ENVSENSE_AGENT=none                    # Disable agent detection
ENVSENSE_AGENT=custom-agent            # Force specific agent
ENVSENSE_ASSUME_HUMAN=1                # Disable agent detection (semantic)
```

#### IDE Detection (New)

```bash
ENVSENSE_IDE=none                      # Disable IDE detection
ENVSENSE_IDE=custom-editor             # Force specific IDE
ENVSENSE_ASSUME_TERMINAL=1             # Disable IDE detection (semantic)
```

#### CI Detection (New)

```bash
ENVSENSE_CI=none                       # Disable CI detection
ENVSENSE_CI=custom-ci                  # Force specific CI system
ENVSENSE_ASSUME_LOCAL=1                # Disable CI detection (semantic)
```

#### Future Extensibility

```bash
ENVSENSE_CONTAINER=none                # Disable container detection
ENVSENSE_CONTAINER=custom-container    # Force specific container
ENVSENSE_ASSUME_BARE_METAL=1           # Disable container detection
```

## Use Cases and Examples

### Testing Scenarios

#### Test Custom Environments

```bash
# Test with custom IDE not in mappings
ENVSENSE_IDE=my-custom-editor ./test-script

# Test with proprietary CI system
ENVSENSE_CI=company-internal-ci ./build-script

# Test with custom agent
ENVSENSE_AGENT=experimental-agent ./agent-test
```

#### Test Edge Cases

```bash
# Test behavior when no IDE is detected
ENVSENSE_IDE=none ./test-terminal-only

# Test behavior when no CI is detected
ENVSENSE_CI=none ./test-local-development

# Test behavior when no agent is detected
ENVSENSE_AGENT=none ./test-human-interaction
```

#### Test Combinations

```bash
# Test agent in custom IDE
ENVSENSE_AGENT=cursor ENVSENSE_IDE=custom-editor ./test

# Test local development with custom agent
ENVSENSE_CI=none ENVSENSE_AGENT=aider ./local-dev-test

# Test pure terminal environment
ENVSENSE_ASSUME_TERMINAL=1 ENVSENSE_ASSUME_LOCAL=1 ENVSENSE_ASSUME_HUMAN=1 ./terminal-test
```

### Debugging Scenarios

#### Isolate Detection Issues

```bash
# Disable IDE detection to test other detectors
ENVSENSE_IDE=none ./debug-script

# Disable CI detection when running locally in CI-like environment
ENVSENSE_CI=none ./local-debug

# Force specific detection to test downstream behavior
ENVSENSE_AGENT=cursor ./test-cursor-specific-features
```

#### Environment Troubleshooting

```bash
# Check what happens without any detection
ENVSENSE_ASSUME_HUMAN=1 ENVSENSE_ASSUME_TERMINAL=1 ENVSENSE_ASSUME_LOCAL=1 ./minimal-test

# Test with only one type of detection
ENVSENSE_CI=none ENVSENSE_AGENT=none ./ide-only-test
```

### Production Scenarios

#### Custom Environments

```bash
# Company with proprietary CI system
export ENVSENSE_CI=jenkins-enterprise
./deployment-script

# Custom IDE integration
export ENVSENSE_IDE=company-editor
./development-workflow

# Specialized agent setup
export ENVSENSE_AGENT=company-ai-assistant
./ai-assisted-development
```

#### Environment Normalization

```bash
# Force local development mode in CI for testing
ENVSENSE_ASSUME_LOCAL=1 ./test-local-behavior

# Force terminal mode for consistent scripting
ENVSENSE_ASSUME_TERMINAL=1 ./automated-script
```

## Implementation Design

### Core Override Function

```rust
// src/detectors/overrides.rs (new file)
use crate::detectors::{EnvSnapshot, confidence::HIGH};
use crate::schema::Evidence;

#[derive(Debug, Clone)]
pub enum OverrideResult {
    Disable,                    // Detection should be disabled
    Force(String),             // Force specific value
    NoOverride,                // No override, use normal detection
}

pub fn check_detector_overrides(
    snap: &EnvSnapshot,
    detector_type: &str,
) -> OverrideResult {
    let detector_upper = detector_type.to_uppercase();
    let direct_key = format!("ENVSENSE_{}", detector_upper);
    let assume_key = get_assume_key(detector_type);

    // Check assume override first (disable detection)
    if let Some(assume_value) = snap.get_env(&assume_key) {
        if assume_value == "1" {
            return OverrideResult::Disable;
        }
    }

    // Check direct override
    if let Some(override_value) = snap.get_env(&direct_key) {
        if override_value == "none" {
            return OverrideResult::Disable;
        } else {
            return OverrideResult::Force(override_value);
        }
    }

    OverrideResult::NoOverride
}

fn get_assume_key(detector_type: &str) -> String {
    let assume_mode = match detector_type {
        "agent" => "HUMAN",
        "ide" => "TERMINAL",
        "ci" => "LOCAL",
        "container" => "BARE_METAL",
        _ => "NONE",
    };
    format!("ENVSENSE_ASSUME_{}", assume_mode)
}

pub fn create_override_evidence(
    detector_type: &str,
    override_value: &str,
    env_key: &str,
) -> Evidence {
    Evidence::env_var(env_key, override_value)
        .with_supports(vec![
            detector_type.into(),
            format!("{}_id", detector_type).into(),
        ])
        .with_confidence(HIGH)
}
```

### Integration Pattern

```rust
// Pattern for all declarative detectors
fn detect_with_overrides(&self, snap: &EnvSnapshot) -> (Option<String>, f32, Vec<Evidence>) {
    match check_detector_overrides(snap, "ide") { // or "ci", "agent"
        OverrideResult::Disable => {
            (None, 0.0, vec![])
        }
        OverrideResult::Force(value) => {
            let evidence = vec![create_override_evidence(
                "ide",
                &value,
                "ENVSENSE_IDE"
            )];
            (Some(value), HIGH, evidence)
        }
        OverrideResult::NoOverride => {
            // Proceed with normal detection logic
            self.detect_normal(snap)
        }
    }
}
```

### Detector Integration Examples

#### IDE Detector

```rust
impl DeclarativeIdeDetector {
    fn detect_ide(&self, snap: &EnvSnapshot) -> (Option<String>, f32, Vec<Evidence>) {
        // Check overrides first
        match check_detector_overrides(snap, "ide") {
            OverrideResult::Disable => return (None, 0.0, vec![]),
            OverrideResult::Force(value) => {
                let evidence = vec![create_override_evidence("ide", &value, "ENVSENSE_IDE")];
                return (Some(value), HIGH, evidence);
            }
            OverrideResult::NoOverride => {
                // Continue with normal detection...
            }
        }

        // Existing detection logic
        let mappings = get_ide_mappings();
        // ... rest of detection
    }
}
```

#### CI Detector

```rust
impl DeclarativeCiDetector {
    fn detect_ci(&self, snap: &EnvSnapshot) -> (Option<String>, f32, Vec<Evidence>) {
        // Check overrides first
        match check_detector_overrides(snap, "ci") {
            OverrideResult::Disable => return (None, 0.0, vec![]),
            OverrideResult::Force(value) => {
                let evidence = if Self::should_generate_evidence() {
                    vec![create_override_evidence("ci", &value, "ENVSENSE_CI")]
                } else {
                    vec![] // CI detector doesn't generate evidence
                };
                return (Some(value), HIGH, evidence);
            }
            OverrideResult::NoOverride => {
                // Continue with normal detection...
            }
        }

        // Existing detection logic
        let mappings = get_ci_mappings();
        // ... rest of detection
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod override_tests {
    use super::*;

    #[test]
    fn test_ide_override_disable() {
        let snapshot = create_env_snapshot(vec![
            ("ENVSENSE_IDE", "none"),
            ("TERM_PROGRAM", "vscode"), // Should be ignored
        ]);

        let detector = DeclarativeIdeDetector::new();
        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.is_empty());
        assert!(detection.facets_patch.is_empty());
        assert_eq!(detection.confidence, 0.0);
    }

    #[test]
    fn test_ide_override_force() {
        let snapshot = create_env_snapshot(vec![
            ("ENVSENSE_IDE", "my-custom-editor"),
        ]);

        let detector = DeclarativeIdeDetector::new();
        let detection = detector.detect(&snapshot);

        assert_eq!(detection.contexts_add, vec!["ide"]);
        assert_eq!(
            detection.facets_patch.get("ide_id").unwrap(),
            &json!("my-custom-editor")
        );
        assert_eq!(detection.confidence, HIGH);
    }

    #[test]
    fn test_assume_terminal_override() {
        let snapshot = create_env_snapshot(vec![
            ("ENVSENSE_ASSUME_TERMINAL", "1"),
            ("TERM_PROGRAM", "vscode"), // Should be ignored
        ]);

        let detector = DeclarativeIdeDetector::new();
        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.is_empty());
        assert_eq!(detection.confidence, 0.0);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_multiple_overrides() {
    let snapshot = create_env_snapshot(vec![
        ("ENVSENSE_AGENT", "custom-agent"),
        ("ENVSENSE_IDE", "custom-editor"),
        ("ENVSENSE_CI", "none"),
        ("GITHUB_ACTIONS", "true"), // Should be ignored due to CI override
    ]);

    // Test that each detector respects its override
    let agent_detector = DeclarativeAgentDetector::new();
    let agent_detection = agent_detector.detect(&snapshot);
    assert_eq!(agent_detection.facets_patch.get("agent_id").unwrap(), &json!("custom-agent"));

    let ide_detector = DeclarativeIdeDetector::new();
    let ide_detection = ide_detector.detect(&snapshot);
    assert_eq!(ide_detection.facets_patch.get("ide_id").unwrap(), &json!("custom-editor"));

    let ci_detector = DeclarativeCiDetector::new();
    let ci_detection = ci_detector.detect(&snapshot);
    assert!(ci_detection.contexts_add.is_empty()); // CI disabled
}
```

### CLI Integration Tests

```bash
#!/bin/bash
# Test override behavior in CLI

# Test IDE override
ENVSENSE_IDE=custom-editor envsense info --json | jq '.facets.ide_id' | grep -q "custom-editor"

# Test CI override disable
ENVSENSE_CI=none GITHUB_ACTIONS=true envsense check ci
test $? -eq 1  # Should fail because CI detection is disabled

# Test assume overrides
ENVSENSE_ASSUME_LOCAL=1 GITHUB_ACTIONS=true envsense info --json | jq '.contexts' | grep -qv "ci"
```

## Documentation Updates

### README.md Updates

Add section on override system:

````markdown
## Override System

Envsense supports environment variable overrides for testing and custom
environments:

### Disable Detection

```bash
ENVSENSE_AGENT=none          # Disable agent detection
ENVSENSE_IDE=none            # Disable IDE detection
ENVSENSE_CI=none             # Disable CI detection
```
````

### Force Specific Values

```bash
ENVSENSE_AGENT=my-agent      # Force specific agent
ENVSENSE_IDE=my-editor       # Force specific IDE
ENVSENSE_CI=my-ci            # Force specific CI
```

### Semantic Overrides

```bash
ENVSENSE_ASSUME_HUMAN=1      # Disable agent detection
ENVSENSE_ASSUME_TERMINAL=1   # Disable IDE detection
ENVSENSE_ASSUME_LOCAL=1      # Disable CI detection
```

````

### CLI Help Updates

```bash
envsense --help
# Add section:
#
# OVERRIDE ENVIRONMENT VARIABLES:
#   ENVSENSE_AGENT=<value>       Override agent detection
#   ENVSENSE_IDE=<value>         Override IDE detection
#   ENVSENSE_CI=<value>          Override CI detection
#   ENVSENSE_ASSUME_HUMAN=1      Disable agent detection
#   ENVSENSE_ASSUME_TERMINAL=1   Disable IDE detection
#   ENVSENSE_ASSUME_LOCAL=1      Disable CI detection
````

## Migration Plan

### Phase 1: Core Infrastructure

1. Create `src/detectors/overrides.rs`
2. Implement core override functions
3. Add unit tests for override logic

### Phase 2: IDE Integration

1. Add override support to `DeclarativeIdeDetector`
2. Add IDE-specific tests
3. Update documentation

### Phase 3: CI Integration

1. Add override support to `DeclarativeCiDetector`
2. Add CI-specific tests
3. Update CLI integration tests

### Phase 4: Agent Migration

1. Migrate existing agent override logic to use new system
2. Ensure backward compatibility
3. Update agent tests

### Phase 5: Documentation and Examples

1. Update README.md
2. Add CLI help text
3. Create usage examples
4. Update integration tests

## Backward Compatibility

### Existing Variables (Maintained)

- `ENVSENSE_ASSUME_HUMAN=1` → Continue to work exactly as before
- `ENVSENSE_AGENT=none` → Continue to work exactly as before
- `ENVSENSE_AGENT=<value>` → Continue to work exactly as before

### New Variables (Additive)

- All new override variables are additive
- No existing behavior changes
- No breaking changes to API or CLI

## Future Enhancements

### Configuration Files

```yaml
# .envsense.yml
overrides:
  agent: custom-agent
  ide: custom-editor
  ci: none
```

### Environment Profiles

```bash
ENVSENSE_PROFILE=testing  # Load testing.yml profile
ENVSENSE_PROFILE=production  # Load production.yml profile
```

### Conditional Overrides

```bash
ENVSENSE_IDE_IF_CI=none  # Disable IDE detection only in CI
ENVSENSE_AGENT_IF_LOCAL=none  # Disable agent detection only locally
```
