# Phase 3 Implementation Plan: Dependency Injection for Testability

## Overview

This document outlines the detailed implementation plan for Phase 3 of the EnvSense simplification proposal. The goal is to replace environment variable overrides with proper dependency injection to improve testability and reduce complexity.



## Current State Analysis

### Current TTY Detection Implementation

The current implementation in `src/detectors/mod.rs` uses environment variable overrides:

```rust
impl EnvSnapshot {
    pub fn current() -> Self {
        use std::io::IsTerminal;

        let env_vars: HashMap<String, String> = std::env::vars().collect();

        // Allow overriding TTY detection for testing
        let is_tty_stdin = env_vars
            .get("ENVSENSE_TTY_STDIN")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or_else(|| std::io::stdin().is_terminal());

        let is_tty_stdout = env_vars
            .get("ENVSENSE_TTY_STDOUT")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or_else(|| std::io::stdout().is_terminal());

        let is_tty_stderr = env_vars
            .get("ENVSENSE_TTY_STDERR")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or_else(|| std::io::stderr().is_terminal());

        Self {
            env_vars,
            is_tty_stdin,
            is_tty_stdout,
            is_tty_stderr,
        }
    }
}
```

### Problems with Current Approach

1. **Environment Variable Pollution**: Test environment variables leak into the global environment
2. **Complex Override Logic**: Parsing and fallback logic adds complexity
3. **Poor Testability**: Hard to test individual components in isolation
4. **Runtime Dependencies**: TTY detection is tied to system calls at runtime
5. **Inconsistent Testing**: Different test approaches (unit vs integration) have different behaviors

### Current Test Usage

Tests currently use environment variable overrides:

```bash
# tests/snapshots/plain_tty.env
TERM=xterm-256color
ENVSENSE_TTY_STDIN=true
ENVSENSE_TTY_STDOUT=false
ENVSENSE_TTY_STDERR=false
ENVSENSE_COLOR_LEVEL=none
ENVSENSE_SUPPORTS_HYPERLINKS=false
```

## Proposed Solution: Dependency Injection

### Design Overview

Replace environment variable overrides with an enum-based dependency injection system:

1. **TtyDetector Enum**: Simple enum with Real and Mock variants for optimal performance
2. **Real Variant**: Production implementation using system calls
3. **Mock Variant**: Test implementation with configurable values
4. **Updated EnvSnapshot**: Uses dependency injection instead of environment variables

**Key Benefits**: Using an enum eliminates dynamic dispatch overhead and provides a simple, performant implementation while maintaining all the benefits of dependency injection.

### Architecture Changes

#### 1. Create TtyDetector Enum

**File**: `src/detectors/tty.rs`

```rust
/// Enum-based TTY detector for optimal performance and simple implementation
#[derive(Debug, Clone)]
pub enum TtyDetector {
    Real,
    Mock {
        stdin: bool,
        stdout: bool,
        stderr: bool,
    },
}

impl TtyDetector {
    /// Create a real TTY detector that uses system calls
    pub fn real() -> Self {
        Self::Real
    }
    
    /// Create a mock TTY detector with specified values
    pub fn mock(stdin: bool, stdout: bool, stderr: bool) -> Self {
        Self::Mock { stdin, stdout, stderr }
    }
    
    /// Create a mock TTY detector for all TTY streams
    pub fn mock_all_tty() -> Self {
        Self::Mock { stdin: true, stdout: true, stderr: true }
    }
    
    /// Create a mock TTY detector for no TTY streams
    pub fn mock_no_tty() -> Self {
        Self::Mock { stdin: false, stdout: false, stderr: false }
    }
    
    /// Create a mock TTY detector for piped I/O (stdin TTY, stdout/stderr not)
    pub fn mock_piped_io() -> Self {
        Self::Mock { stdin: true, stdout: false, stderr: false }
    }
    
    /// Check if stdin is a TTY
    pub fn is_tty_stdin(&self) -> bool {
        match self {
            Self::Real => {
                use std::io::IsTerminal;
                std::io::stdin().is_terminal()
            }
            Self::Mock { stdin, .. } => *stdin,
        }
    }
    
    /// Check if stdout is a TTY
    pub fn is_tty_stdout(&self) -> bool {
        match self {
            Self::Real => {
                use std::io::IsTerminal;
                std::io::stdout().is_terminal()
            }
            Self::Mock { stdout, .. } => *stdout,
        }
    }
    
    /// Check if stderr is a TTY
    pub fn is_tty_stderr(&self) -> bool {
        match self {
            Self::Real => {
                use std::io::IsTerminal;
                std::io::stderr().is_terminal()
            }
            Self::Mock { stderr, .. } => *stderr,
        }
    }
}
```

#### 2. Update EnvSnapshot

**File**: `src/detectors/mod.rs`

```rust
use crate::detectors::tty::TtyDetector;

#[derive(Debug, Clone)]
pub struct EnvSnapshot {
    pub env_vars: HashMap<String, String>,
    pub tty_detector: TtyDetector,
}

impl EnvSnapshot {
    /// Create snapshot with real TTY detection for production use
    pub fn current() -> Self {
        Self {
            env_vars: std::env::vars().collect(),
            tty_detector: TtyDetector::real(),
        }
    }
    
    /// Create snapshot with mock TTY detection for testing
    pub fn for_testing(
        env_vars: HashMap<String, String>, 
        tty_detector: TtyDetector
    ) -> Self {
        Self { env_vars, tty_detector }
    }
    
    /// Create snapshot with mock TTY detection using convenience constructor
    pub fn with_mock_tty(
        env_vars: HashMap<String, String>,
        stdin: bool,
        stdout: bool,
        stderr: bool,
    ) -> Self {
        Self {
            env_vars,
            tty_detector: TtyDetector::mock(stdin, stdout, stderr),
        }
    }
    
    /// Convenience methods that delegate to the TTY detector
    pub fn is_tty_stdin(&self) -> bool {
        self.tty_detector.is_tty_stdin()
    }
    
    pub fn is_tty_stdout(&self) -> bool {
        self.tty_detector.is_tty_stdout()
    }
    
    pub fn is_tty_stderr(&self) -> bool {
        self.tty_detector.is_tty_stderr()
    }
    
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.env_vars.get(key)
    }
}
```

#### 3. Update Detector Module Exports

**File**: `src/detectors/mod.rs`

```rust
pub mod tty;
pub use tty::TtyDetector;
```

#### 4. Update Terminal Detector

**File**: `src/detectors/terminal.rs`

```rust
use crate::detectors::{Detection, Detector, EnvSnapshot, confidence::TERMINAL};
use crate::schema::Evidence;
use crate::traits::terminal::ColorLevel;
use serde_json::json;

pub struct TerminalDetector;

impl TerminalDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Detector for TerminalDetector {
    fn name(&self) -> &'static str {
        "terminal"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        // TTY detection is always reliable
        detection.confidence = TERMINAL;

        // Use TTY values from snapshot (now via dependency injection)
        let is_interactive = snap.is_tty_stdin() && snap.is_tty_stdout();

        // Detect color level and hyperlinks support, but allow override
        let color_level = if let Some(override_color) = snap.get_env("ENVSENSE_COLOR_LEVEL") {
            match override_color.as_str() {
                "none" => ColorLevel::None,
                "ansi16" => ColorLevel::Ansi16,
                "ansi256" => ColorLevel::Ansi256,
                "truecolor" => ColorLevel::Truecolor,
                _ => ColorLevel::None,
            }
        } else {
            // Use runtime detection
            let level = supports_color::on(supports_color::Stream::Stdout);
            match level {
                Some(l) => {
                    if l.has_16m {
                        ColorLevel::Truecolor
                    } else if l.has_256 {
                        ColorLevel::Ansi256
                    } else if l.has_basic {
                        ColorLevel::Ansi16
                    } else {
                        ColorLevel::None
                    }
                }
                None => ColorLevel::None,
            }
        };

        let supports_hyperlinks = if let Some(override_hyperlinks) = snap.get_env("ENVSENSE_SUPPORTS_HYPERLINKS") {
            override_hyperlinks.parse::<bool>().unwrap_or(false)
        } else {
            // Use runtime detection
            supports_hyperlinks::on(supports_hyperlinks::Stream::Stdout)
        };

        // Set traits based on TTY detection
        detection.traits_patch.insert("is_tty_stdin".to_string(), json!(snap.is_tty_stdin()));
        detection.traits_patch.insert("is_tty_stdout".to_string(), json!(snap.is_tty_stdout()));
        detection.traits_patch.insert("is_tty_stderr".to_string(), json!(snap.is_tty_stderr()));
        detection.traits_patch.insert("is_piped_stdin".to_string(), json!(!snap.is_tty_stdin()));
        detection.traits_patch.insert("is_piped_stdout".to_string(), json!(!snap.is_tty_stdout()));
        detection.traits_patch.insert("is_interactive".to_string(), json!(is_interactive));
        detection.traits_patch.insert("supports_hyperlinks".to_string(), json!(supports_hyperlinks));

        // Add evidence
        detection.evidence.push(Evidence {
            signal: crate::schema::Signal::Tty,
            key: "stdin_tty".to_string(),
            value: Some(snap.is_tty_stdin().to_string()),
            supports: vec!["terminal".to_string()],
            confidence: TERMINAL,
        });

        detection.evidence.push(Evidence {
            signal: crate::schema::Signal::Tty,
            key: "stdout_tty".to_string(),
            value: Some(snap.is_tty_stdout().to_string()),
            supports: vec!["terminal".to_string()],
            confidence: TERMINAL,
        });

        detection.evidence.push(Evidence {
            signal: crate::schema::Signal::Tty,
            key: "stderr_tty".to_string(),
            value: Some(snap.is_tty_stderr().to_string()),
            supports: vec!["terminal".to_string()],
            confidence: TERMINAL,
        });

        detection
    }
}
```

## Implementation Steps

### Step 1: Create TTY Module (Day 1)

1. **Create `src/detectors/tty.rs`**
   - Implement `TtyDetector` enum with Real and Mock variants
   - Add convenience constructors for common scenarios
   - Implement TTY detection methods

2. **Update `src/detectors/mod.rs`**
   - Add `pub mod tty;` and exports
   - Update `EnvSnapshot` to use dependency injection
   - Remove environment variable override logic
   - Add convenience methods for TTY detection

3. **Test the new module**
   - Create unit tests for `TtyDetector` enum
   - Verify Real variant works correctly
   - Verify Mock variant can be configured

### Step 2: Update Terminal Detector (Day 2)

1. **Update `src/detectors/terminal.rs`**
   - Remove direct TTY system calls
   - Use `EnvSnapshot` TTY methods
   - Keep environment variable overrides for color/hyperlinks (for now)

2. **Test terminal detector**
   - Verify it works with Real TTY detection
   - Verify it works with Mock TTY detection
   - Ensure all existing functionality is preserved

### Step 3: Update Tests (Day 3)

1. **Update unit tests**
   - Replace environment variable setup with `TtyDetector::mock()`
   - Update test helpers to use new `EnvSnapshot` constructors
   - Ensure all existing tests pass

2. **Update integration tests**
   - Modify baseline comparison script to use new approach
   - Update snapshot tests to use dependency injection
   - Verify CI tests still work correctly

### Step 4: Update CLI and Engine (Day 4)

1. **Update `src/engine.rs`**
   - Ensure `DetectionEngine::detect()` uses `EnvSnapshot::current()`
   - Verify `detect_from_snapshot()` works with new approach

2. **Update `src/main.rs`**
   - Ensure CLI uses production TTY detection
   - Verify all CLI functionality works

### Step 5: Clean Up and Documentation (Day 5)

1. **Remove environment variable overrides**
   - Remove `ENVSENSE_TTY_*` environment variables
   - Update documentation to reflect new approach
   - Update scripts that set these variables

2. **Update documentation**
   - Document new dependency injection approach
   - Update testing guide
   - Update debugging guide

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::detectors::{EnvSnapshot, TtyDetector};

    #[test]
    fn test_tty_detector_enum() {
        let detector = TtyDetector::mock(true, false, true);
        assert!(detector.is_tty_stdin());
        assert!(!detector.is_tty_stdout());
        assert!(detector.is_tty_stderr());
    }

    #[test]
    fn test_tty_detector_convenience_methods() {
        let all_tty = TtyDetector::mock_all_tty();
        assert!(all_tty.is_tty_stdin());
        assert!(all_tty.is_tty_stdout());
        assert!(all_tty.is_tty_stderr());

        let no_tty = TtyDetector::mock_no_tty();
        assert!(!no_tty.is_tty_stdin());
        assert!(!no_tty.is_tty_stdout());
        assert!(!no_tty.is_tty_stderr());

        let piped = TtyDetector::mock_piped_io();
        assert!(piped.is_tty_stdin());
        assert!(!piped.is_tty_stdout());
        assert!(!piped.is_tty_stderr());
    }

    #[test]
    fn test_env_snapshot_with_mock_tty() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TERM".to_string(), "xterm-256color".to_string());
        
        let snapshot = EnvSnapshot::with_mock_tty(env_vars, true, false, false);
        
        assert!(snapshot.is_tty_stdin());
        assert!(!snapshot.is_tty_stdout());
        assert!(!snapshot.is_tty_stderr());
        assert_eq!(snapshot.get_env("TERM"), Some(&"xterm-256color".to_string()));
    }

    #[test]
    fn test_terminal_detector_with_mock_tty() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TERM".to_string(), "xterm-256color".to_string());
        
        let snapshot = EnvSnapshot::with_mock_tty(env_vars, true, false, false);
        let detector = TerminalDetector::new();
        let detection = detector.detect(&snapshot);
        
        assert_eq!(detection.traits_patch["is_tty_stdin"], json!(true));
        assert_eq!(detection.traits_patch["is_tty_stdout"], json!(false));
        assert_eq!(detection.traits_patch["is_interactive"], json!(false));
    }
}
```

### Integration Tests

```rust
#[test]
fn test_baseline_scenarios_with_dependency_injection() {
    // Test each baseline scenario using TtyDetector::mock()
    let scenarios = vec![
        ("plain_tty", true, false, false),
        ("piped_io", true, false, false),
        ("github_actions", true, false, false),
        ("gitlab_ci", true, false, false),
    ];
    
    for (scenario_name, stdin, stdout, stderr) in scenarios {
        let env_vars = load_scenario_env(scenario_name);
        let snapshot = EnvSnapshot::with_mock_tty(env_vars, stdin, stdout, stderr);
        
        let engine = DetectionEngine::with_all_detectors();
        let result = engine.detect_from_snapshot(&snapshot);
        
        // Verify result matches expected baseline
        assert_baseline_matches(scenario_name, &result);
    }
}
```

## Migration Strategy

### Backward Compatibility

1. **Gradual Migration**: Keep environment variable overrides working during transition
2. **Feature Flag**: Add feature flag to enable/disable dependency injection
3. **Fallback Logic**: Fall back to environment variables if dependency injection fails

### Rollout Plan

1. **Week 1**: Implement dependency injection alongside existing approach
2. **Week 2**: Update tests to use new approach
3. **Week 3**: Enable dependency injection by default
4. **Week 4**: Remove environment variable overrides



## Risk Assessment

### Low Risk
- **Enum Implementation**: Simple, well-understood Rust pattern
- **Mock Implementation**: Direct boolean fields, no complexity
- **Unit Tests**: Isolated testing with clear inputs/outputs
- **Performance**: No dynamic dispatch, optimal performance

### Medium Risk
- **Integration Tests**: May reveal edge cases in real environments
- **Migration Effort**: Significant work to update all tests and scripts
- **CI Compatibility**: Need to ensure CI environments work correctly

### Mitigation Strategies

1. **Comprehensive Testing**: Test all scenarios with both real and mock detectors
2. **Gradual Rollout**: Implement alongside existing approach initially
3. **Backward Compatibility**: Keep environment variable approach during transition
4. **Feature Flag**: Enable/disable dependency injection during rollout

## Success Metrics

1. **Code Complexity Reduction**: Remove 20+ lines of environment variable parsing logic
2. **Test Clarity**: Tests become more explicit about TTY state
3. **Performance**: Optimal performance with enum-based approach (no dynamic dispatch)
4. **Maintainability**: Easier to test individual components
5. **Documentation**: Clearer testing patterns for contributors
6. **Migration Success**: All existing tests pass with new approach
7. **Backward Compatibility**: No breaking changes during transition

## Future Enhancements

### Phase 3.5: Extended Dependency Injection

After successful implementation, consider extending to other system dependencies:

1. **File System Detection**: Abstract file system operations
2. **Process Detection**: Abstract process tree traversal
3. **Network Detection**: Abstract network interface queries
4. **Environment Variables**: Abstract environment variable access

### Benefits of Extended DI

1. **Better Testability**: All system dependencies become mockable
2. **Platform Independence**: Easier to test cross-platform behavior
3. **Performance Testing**: Mock slow operations for faster tests
4. **Error Testing**: Simulate system failures and edge cases

## Conclusion

Phase 3 dependency injection will significantly improve the testability and maintainability of the EnvSense codebase. By replacing environment variable overrides with proper dependency injection using an enum-based approach, we eliminate complexity while making the code more modular and easier to test.

### Key Benefits

1. **Optimal Performance**: Enum-based design eliminates dynamic dispatch overhead
2. **Simplified Implementation**: Clean, straightforward enum with Real and Mock variants
3. **Better Testability**: Complete isolation through mock objects
4. **Reduced Complexity**: Remove environment variable parsing logic
5. **Maintainable Architecture**: Clear separation of concerns

### Recommendation

**Proceed with Phase 3** implementation:

1. **Implementation**: Gradual rollout with thorough testing and backward compatibility
2. **Timeline**: 5-day implementation plan with comprehensive testing
3. **Risk Mitigation**: Feature flags and fallback mechanisms during transition

The implementation plan provides a clear path forward with minimal risk and maximum benefit. The enum-based approach delivers optimal performance while maintaining all the architectural benefits of dependency injection.
