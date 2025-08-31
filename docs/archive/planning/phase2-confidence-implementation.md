# Phase 2: Confidence Scoring Simplification Implementation Plan

## Overview

This document details the implementation plan for Phase 2 of the EnvSense
simplification proposal: **Confidence Scoring Simplification**. The goal is to
replace arbitrary hardcoded confidence values with clear, meaningful constants
that have documented reasoning.

## Current State Analysis

### Problems Identified

1. **Inconsistent Values**: Different detectors use different hardcoded values:
   - Terminal detector: `1.0` (always reliable)
   - IDE detector: `0.95`
   - CI detector: `0.9`
   - Agent detector: Various values (`0.8`, `0.9`, `0.95`, `1.0`)

2. **No Clear Reasoning**: Values appear arbitrary with no documented rationale

3. **Limited Usage**: Confidence is only used in individual `Evidence` items,
   not in final output schema

4. **Default Value**: Detection struct defaults to `0.0` confidence, which is
   meaningless

### Current Confidence Usage

```rust
// src/detectors/terminal.rs
detection.confidence = 1.0; // Terminal detection is always reliable

// src/detectors/ide.rs
detection.confidence = 0.95;

// src/detectors/ci.rs
detection.confidence = 0.9; // High confidence for CI detection

// src/detectors/agent.rs
detection.agent.confidence = 0.95; // Various hardcoded values
detection.agent.confidence = 0.9;
detection.agent.confidence = 0.8;
```

## Proposed Solution

### Option 1: Confidence Constants (Recommended)

Replace hardcoded values with well-documented constants that have clear
reasoning.

#### Confidence Level Definitions

```rust
// src/detectors/mod.rs
pub mod confidence {
    /// Direct environment variable match (e.g., TERM_PROGRAM=vscode)
    /// Used when we have a clear, unambiguous signal from the environment
    /// Examples: CI=true, TERM_PROGRAM=vscode, CURSOR_AGENT=1
    pub const HIGH: f32 = 1.0;

    /// Inferred from context (e.g., multiple environment variables)
    /// Used when we have strong but indirect evidence
    /// Examples: Multiple AIDER_* variables, SANDBOX_* variables
    pub const MEDIUM: f32 = 0.8;

    /// Heuristic detection (e.g., file path patterns, process names)
    /// Used when we have weak or circumstantial evidence
    /// Examples: Process name patterns, file system markers
    pub const LOW: f32 = 0.6;

    /// Terminal detection (always reliable)
    /// Used for TTY detection which is always accurate
    /// Examples: isatty() calls, terminal capability detection
    pub const TERMINAL: f32 = 1.0;
}
```

#### Benefits

- **Clear Reasoning**: Each confidence level has documented purpose
- **Consistency**: All detectors use the same values
- **Maintainability**: Easy to understand and modify
- **Backward Compatible**: Doesn't change the schema
- **Documentation**: Self-documenting code with clear intent

### Option 2: Optional Confidence (Alternative)

Make confidence optional for simple boolean detections.

```rust
pub struct Detection {
    pub contexts_add: Vec<String>,
    pub traits_patch: HashMap<String, serde_json::Value>,
    pub facets_patch: HashMap<String, serde_json::Value>,
    pub evidence: Vec<Evidence>,
    pub confidence: Option<f32>, // Optional for simple cases
}
```

#### Benefits

- **Simpler**: Only requires confidence when meaningful
- **Reduces Noise**: Less clutter in output for simple detections

#### Drawbacks

- **Breaking Change**: Changes the schema
- **Complexity**: Requires updating all consumers
- **Merging Logic**: More complex engine merging

## Implementation Plan

### Phase 2A: Add Confidence Constants (Week 1)

#### Step 1: Create Confidence Module

**File**: `src/detectors/mod.rs`

```rust
/// Confidence levels for detection results
///
/// These constants provide clear, meaningful confidence values
/// with documented reasoning for when to use each level.
pub mod confidence {
    /// Direct environment variable match (e.g., TERM_PROGRAM=vscode)
    ///
    /// Used when we have a clear, unambiguous signal from the environment.
    /// This is the highest confidence level for environment-based detection.
    ///
    /// Examples:
    /// - `CI=true` - Direct CI environment indicator
    /// - `TERM_PROGRAM=vscode` - Direct IDE environment indicator
    /// - `CURSOR_AGENT=1` - Direct agent environment indicator
    pub const HIGH: f32 = 1.0;

    /// Inferred from context (e.g., multiple environment variables)
    ///
    /// Used when we have strong but indirect evidence that requires
    /// interpretation or combination of multiple signals.
    ///
    /// Examples:
    /// - Multiple `AIDER_*` variables indicating Aider agent
    /// - Multiple `SANDBOX_*` variables indicating sandboxed environment
    /// - Combination of SSH variables indicating remote session
    pub const MEDIUM: f32 = 0.8;

    /// Heuristic detection (e.g., file path patterns, process names)
    ///
    /// Used when we have weak or circumstantial evidence that requires
    /// pattern matching or heuristic analysis.
    ///
    /// Examples:
    /// - Process name patterns (e.g., "cursor" in process tree)
    /// - File system markers (e.g., `.vscode` directory)
    /// - Network connection patterns
    pub const LOW: f32 = 0.6;

    /// Terminal detection (always reliable)
    ///
    /// Used for TTY detection which is always accurate because it
    /// directly queries the operating system for terminal capabilities.
    ///
    /// Examples:
    /// - `isatty()` calls for stdin/stdout/stderr
    /// - Terminal capability detection
    /// - Color support detection
    pub const TERMINAL: f32 = 1.0;
}
```

#### Step 2: Update Detection Default

**File**: `src/detectors/mod.rs`

```rust
impl Default for Detection {
    fn default() -> Self {
        Self {
            contexts_add: Vec::new(),
            traits_patch: HashMap::new(),
            facets_patch: HashMap::new(),
            evidence: Vec::new(),
            confidence: 0.0, // Keep existing default for now
        }
    }
}
```

### Phase 2B: Update Detector Implementations (Week 2)

#### Step 1: Terminal Detector

**File**: `src/detectors/terminal.rs`

```rust
use crate::detectors::confidence::TERMINAL;

impl Detector for TerminalDetector {
    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        // TTY detection is always reliable
        detection.confidence = TERMINAL;

        // Set TTY traits
        detection.traits_patch.insert("is_tty_stdin".to_string(), json!(snap.is_tty_stdin));
        detection.traits_patch.insert("is_tty_stdout".to_string(), json!(snap.is_tty_stdout));
        detection.traits_patch.insert("is_tty_stderr".to_string(), json!(snap.is_tty_stderr));

        // Add evidence for TTY detection
        detection.evidence.push(
            Evidence::tty_trait("is_tty_stdin", snap.is_tty_stdin)
                .with_supports(vec!["is_tty_stdin".into()])
                .with_confidence(TERMINAL)
        );
        detection.evidence.push(
            Evidence::tty_trait("is_tty_stdout", snap.is_tty_stdout)
                .with_supports(vec!["is_tty_stdout".into()])
                .with_confidence(TERMINAL)
        );
        detection.evidence.push(
            Evidence::tty_trait("is_tty_stderr", snap.is_tty_stderr)
                .with_supports(vec!["is_tty_stderr".into()])
                .with_confidence(TERMINAL)
        );

        detection
    }
}
```

#### Step 2: IDE Detector

**File**: `src/detectors/ide.rs`

```rust
use crate::detectors::confidence::HIGH;

impl Detector for IdeDetector {
    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        if let Some(term_program) = snap.get_env("TERM_PROGRAM")
            && term_program == "vscode"
        {
            detection.contexts_add.push("ide".to_string());
            detection.confidence = HIGH; // Direct env var match

            // Add evidence for IDE context
            detection.evidence.push(
                Evidence::env_var("TERM_PROGRAM", term_program)
                    .with_supports(vec!["ide".into()])
                    .with_confidence(HIGH)
            );

            // Detect specific IDE variant
            if snap.get_env("CURSOR_TRACE_ID").is_some() {
                detection
                    .facets_patch
                    .insert("ide_id".to_string(), json!("cursor"));
                detection.evidence.push(
                    Evidence::env_presence("CURSOR_TRACE_ID")
                        .with_supports(vec!["ide_id".into()])
                        .with_confidence(HIGH) // Direct env var presence
                );
            } else if let Some(version) = snap.get_env("TERM_PROGRAM_VERSION") {
                let ide_id = if version.to_lowercase().contains("insider") {
                    "vscode-insiders"
                } else {
                    "vscode"
                };
                detection
                    .facets_patch
                    .insert("ide_id".to_string(), json!(ide_id));
                detection.evidence.push(
                    Evidence::env_var("TERM_PROGRAM_VERSION", version)
                        .with_supports(vec!["ide_id".into()])
                        .with_confidence(HIGH) // Direct env var match
                );
            }
        }

        detection
    }
}
```

#### Step 3: CI Detector

**File**: `src/detectors/ci.rs`

```rust
use crate::detectors::confidence::HIGH;

impl Detector for CiDetector {
    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        // Use snapshot-based CI detection instead of global environment
        let ci_facet = self.detect_ci_from_snapshot(snap);

        if ci_facet.is_ci {
            detection.contexts_add.push("ci".to_string());
            detection.confidence = HIGH; // Direct env var match (CI=true, GITHUB_ACTIONS, etc.)

            if let Some(vendor) = ci_facet.vendor.clone() {
                detection
                    .facets_patch
                    .insert("ci_id".to_string(), json!(vendor));
            }

            // Generate CI traits
            let traits = ci_traits(&ci_facet);
            for (key, value) in traits {
                detection.traits_patch.insert(key, value);
            }

            // Store the full CI facet data for later use
            detection
                .facets_patch
                .insert("ci".to_string(), json!(ci_facet));
        }

        detection
    }
}
```

#### Step 4: Agent Detector

**File**: `src/detectors/agent.rs`

```rust
use crate::detectors::confidence::{HIGH, MEDIUM, LOW};

// Update the agent detection logic
fn detect_agent(env: &impl EnvReader) -> AgentDetection {
    let mut detection = AgentDetection::default();
    let vars: HashMap<String, String> = env.iter().collect();

    // Handle overrides first
    if vars
        .get("ENVSENSE_ASSUME_HUMAN")
        .map(|v| v == "1")
        .unwrap_or(false)
        || vars
            .get("ENVSENSE_AGENT")
            .map(|v| v == "none")
            .unwrap_or(false)
    {
        detect_editor(&vars, &mut detection.facets);
        detect_replit(&vars, &mut detection, false);
        detect_host(&vars, &mut detection.facets);
        return detection;
    }

    // Direct agent override
    if let Some(slug) = vars.get("ENVSENSE_AGENT") {
        detection.agent.is_agent = true;
        detection.agent.name = Some(slug.clone());
        let (vendor, variant, caps) = descriptor(slug);
        detection.agent.vendor = vendor.map(str::to_string);
        detection.agent.variant = variant.map(str::to_string);
        detection.agent.capabilities = caps;
        detection.agent.confidence = HIGH; // Direct override
    }

    detect_editor(&vars, &mut detection.facets);
    detect_replit(&vars, &mut detection, true);
    detect_host(&vars, &mut detection.facets);

    if detection.agent.name.is_none() {
        // Direct environment variables
        if let Some(v) = vars.get("CURSOR_AGENT") {
            detection.agent.name = Some("cursor".into());
            detection.agent.confidence = HIGH; // Direct env var
            detection.agent.is_agent = true;
            add_raw(&mut detection.agent, "CURSOR_AGENT", v);
        } else if let Some(v) = vars.get("CLINE_ACTIVE") {
            detection.agent.name = Some("cline".into());
            detection.agent.confidence = HIGH; // Direct env var
            detection.agent.is_agent = true;
            add_raw(&mut detection.agent, "CLINE_ACTIVE", v);
        } else if let Some(v) = vars.get("CLAUDECODE") {
            detection.agent.name = Some("claude-code".into());
            detection.agent.confidence = HIGH; // Direct env var
            detection.agent.is_agent = true;
            add_raw(&mut detection.agent, "CLAUDECODE", v);
        }

        // Inferred from context
        else if vars.keys().any(|k| k.starts_with("SANDBOX_")) {
            detection.agent.name = Some("openhands".into());
            detection.agent.confidence = MEDIUM; // Inferred from context
            detection.agent.is_agent = true;
        } else if vars.get("IS_CODE_AGENT").map(|v| v == "1").unwrap_or(false) {
            detection.agent.name = Some("unknown".into());
            detection.agent.confidence = LOW; // Heuristic
            detection.agent.is_agent = true;
        }
    }

    if detection.agent.name.is_none() {
        // Aider detection - inferred from context
        let aider_envs: Vec<&String> = vars.keys().filter(|k| k.starts_with("AIDER_")).collect();
        let aider_detect = vars.contains_key("AIDER_MODEL") || aider_envs.len() >= 2;
        if aider_detect {
            detection.agent.name = Some("aider".into());
            detection.agent.confidence = MEDIUM; // Inferred from context
            detection.agent.is_agent = true;
        }
    }

    detection
}
```

### Phase 2C: Update Evidence Confidence (Week 3)

#### Step 1: Update Evidence Constructors

**File**: `src/schema.rs`

```rust
use crate::detectors::confidence::{HIGH, MEDIUM, TERMINAL};

impl Evidence {
    /// Create evidence from environment variable with value
    ///
    /// Used when we have a direct environment variable match.
    /// Confidence: HIGH (1.0) - Direct env var match
    pub fn env_var(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            signal: Signal::Env,
            key: key.into(),
            value: Some(value.into()),
            supports: Vec::new(),
            confidence: HIGH,
        }
    }

    /// Create evidence from environment variable presence
    ///
    /// Used when we know an environment variable exists but don't capture its value.
    /// Confidence: MEDIUM (0.8) - Inferred from presence
    pub fn env_presence(key: impl Into<String>) -> Self {
        Self {
            signal: Signal::Env,
            key: key.into(),
            value: None,
            supports: Vec::new(),
            confidence: MEDIUM,
        }
    }

    /// Create evidence from TTY trait detection
    ///
    /// Used for terminal capability detection which is always reliable.
    /// Confidence: TERMINAL (1.0) - Always reliable
    pub fn tty_trait(key: impl Into<String>, is_tty: bool) -> Self {
        Self {
            signal: Signal::Tty,
            key: key.into(),
            value: Some(is_tty.to_string()),
            supports: Vec::new(),
            confidence: TERMINAL,
        }
    }

    /// Add support contexts to evidence
    pub fn with_supports(mut self, supports: Vec<String>) -> Self {
        self.supports = supports;
        self
    }

    /// Override confidence level
    ///
    /// Use this sparingly - prefer the default confidence levels
    /// from the evidence constructors.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}
```

### Phase 2D: Update Tests (Week 4)

#### Step 1: Add Confidence Tests

**File**: `tests/confidence_tests.rs` (new file)

```rust
use envsense::schema::EnvSense;
use envsense::detectors::confidence::{HIGH, MEDIUM, LOW, TERMINAL};

#[test]
fn test_confidence_constants() {
    // Verify constants are in valid range
    assert!(HIGH >= 0.0 && HIGH <= 1.0);
    assert!(MEDIUM >= 0.0 && MEDIUM <= 1.0);
    assert!(LOW >= 0.0 && LOW <= 1.0);
    assert!(TERMINAL >= 0.0 && TERMINAL <= 1.0);

    // Verify relative ordering
    assert!(HIGH >= MEDIUM);
    assert!(MEDIUM >= LOW);
    assert_eq!(HIGH, TERMINAL); // Both are 1.0
}

#[test]
fn test_detection_confidence_values() {
    let result = EnvSense::detect();

    // Verify all detections have appropriate confidence levels
    for evidence in &result.evidence {
        match evidence.signal {
            Signal::Env => {
                if evidence.value.is_some() {
                    assert_eq!(evidence.confidence, HIGH,
                        "Direct env var should have HIGH confidence: {}", evidence.key);
                } else {
                    assert_eq!(evidence.confidence, MEDIUM,
                        "Env presence should have MEDIUM confidence: {}", evidence.key);
                }
            }
            Signal::Tty => {
                assert_eq!(evidence.confidence, TERMINAL,
                    "TTY detection should have TERMINAL confidence: {}", evidence.key);
            }
            _ => {
                assert!(evidence.confidence >= LOW && evidence.confidence <= HIGH,
                    "Confidence should be in valid range: {} = {}", evidence.key, evidence.confidence);
            }
        }
    }
}

#[test]
fn test_specific_detector_confidence() {
    // Test terminal detector
    let terminal_detector = TerminalDetector::new();
    let snapshot = EnvSnapshot::current();
    let detection = terminal_detector.detect(&snapshot);
    assert_eq!(detection.confidence, TERMINAL);

    // Test IDE detector with VSCode
    std::env::set_var("TERM_PROGRAM", "vscode");
    let ide_detector = IdeDetector::new();
    let detection = ide_detector.detect(&snapshot);
    if !detection.contexts_add.is_empty() {
        assert_eq!(detection.confidence, HIGH);
    }

    // Clean up
    std::env::remove_var("TERM_PROGRAM");
}

#[test]
fn test_confidence_documentation() {
    // Verify that confidence levels are well-documented
    let result = EnvSense::detect();

    // Check that evidence has appropriate confidence based on signal type
    for evidence in &result.evidence {
        match evidence.signal {
            Signal::Env => {
                if evidence.value.is_some() {
                    // Direct env var match should be HIGH
                    assert!(evidence.confidence >= HIGH,
                        "Direct env var {} should have HIGH confidence, got {}",
                        evidence.key, evidence.confidence);
                } else {
                    // Env presence should be MEDIUM
                    assert!(evidence.confidence >= MEDIUM,
                        "Env presence {} should have MEDIUM confidence, got {}",
                        evidence.key, evidence.confidence);
                }
            }
            Signal::Tty => {
                // TTY detection should always be TERMINAL
                assert_eq!(evidence.confidence, TERMINAL,
                    "TTY detection {} should have TERMINAL confidence", evidence.key);
            }
            _ => {
                // Other signals should be in valid range
                assert!(evidence.confidence >= LOW && evidence.confidence <= HIGH,
                    "Signal {} should have confidence in [LOW, HIGH] range", evidence.signal);
            }
        }
    }
}
```

#### Step 2: Update Existing Tests

Update existing tests to verify confidence values:

```rust
// tests/cli.rs
#[test]
fn test_ide_detection_confidence() {
    let cmd = Command::cargo_bin("envsense").unwrap();
    let output = cmd
        .arg("info")
        .arg("--json")
        .env("TERM_PROGRAM", "vscode")
        .output()
        .unwrap();

    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    // Verify evidence has appropriate confidence
    if let Some(evidence) = result.get("evidence").and_then(|e| e.as_array()) {
        for item in evidence {
            if let Some(signal) = item.get("signal").and_then(|s| s.as_str()) {
                if signal == "env" {
                    let confidence = item.get("confidence").and_then(|c| c.as_f64()).unwrap();
                    if item.get("value").is_some() {
                        assert_eq!(confidence, 1.0); // HIGH
                    } else {
                        assert_eq!(confidence, 0.8); // MEDIUM
                    }
                }
            }
        }
    }
}
```

## Migration Strategy

### Backward Compatibility

This implementation maintains full backward compatibility:

1. **Schema Unchanged**: The `Detection` struct and `Evidence` struct remain
   unchanged
2. **API Unchanged**: All public APIs continue to work as before
3. **Output Unchanged**: JSON output format remains the same
4. **Values Improved**: Confidence values are now more consistent and
   well-documented

### Rollout Plan

1. **Week 1**: Add confidence constants and documentation
2. **Week 2**: Update detector implementations
3. **Week 3**: Update evidence constructors
4. **Week 4**: Add comprehensive tests
5. **Week 5**: Code review and final testing

### Testing Strategy

1. **Unit Tests**: Test each detector individually
2. **Integration Tests**: Test full detection pipeline
3. **Snapshot Tests**: Verify output remains consistent
4. **Confidence Tests**: Verify confidence values are appropriate
5. **Documentation Tests**: Verify confidence reasoning is clear

## Success Metrics

### Quantitative Metrics

1. **Consistency**: 100% of detectors use confidence constants
2. **Test Coverage**: 100% of confidence logic covered by tests
3. **Documentation**: All confidence levels have clear reasoning
4. **Performance**: No regression in detection performance

### Qualitative Metrics

1. **Clarity**: Confidence values are self-documenting
2. **Maintainability**: Easy to understand and modify confidence logic
3. **Consistency**: All similar detections use similar confidence levels
4. **Reasoning**: Clear justification for each confidence level

## Risk Assessment

### Low Risk

- **Adding Constants**: Non-breaking change
- **Updating Detectors**: Contained changes with tests
- **Documentation**: Improves clarity without functional changes

### Medium Risk

- **Value Changes**: Some confidence values may change
- **Consumer Impact**: Consumers may notice different confidence values

### Mitigation Strategies

1. **Comprehensive Testing**: Ensure all changes are well-tested
2. **Documentation**: Clear reasoning for all confidence values
3. **Gradual Rollout**: Implement changes incrementally
4. **Monitoring**: Watch for any issues in production

## Future Considerations

### Potential Enhancements

1. **Dynamic Confidence**: Confidence based on signal strength
2. **Confidence Aggregation**: Combine multiple confidence values
3. **Confidence Thresholds**: Filter results by confidence level
4. **Confidence Reporting**: Include confidence in CLI output

### Alternative Approaches

If the confidence constants approach doesn't meet expectations, we can consider:

1. **Optional Confidence**: Make confidence optional for simple cases
2. **Confidence Ranges**: Use confidence ranges instead of discrete values
3. **Confidence Factors**: Use multiplicative confidence factors

## Conclusion

Phase 2 confidence scoring simplification provides immediate benefits:

1. **Clear Reasoning**: Each confidence level has documented purpose
2. **Consistency**: All detectors use the same confidence values
3. **Maintainability**: Easy to understand and modify
4. **Backward Compatibility**: No breaking changes to schema or API

The implementation plan provides a clear roadmap for achieving these benefits
while minimizing risk and maintaining quality.
