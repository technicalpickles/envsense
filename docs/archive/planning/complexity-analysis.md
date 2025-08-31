# Complexity Analysis: EnvSense Refactored Codebase

This document provides a detailed technical analysis of the complexity
introduced by the refactoring, supporting the simplification proposal.

## Current Architecture Overview

The refactoring transformed a monolithic detection system into a pluggable
architecture:

```
Before: schema.rs (223 lines) - Monolithic detection logic
After:  Multiple modules with clear separation of concerns
```

### New Architecture Components

1. **`src/detectors/mod.rs`** - Detector trait and EnvSnapshot
2. **`src/engine.rs`** - DetectionEngine for orchestration
3. **`src/detectors/{ide,ci,agent,terminal}.rs`** - Specialized detectors
4. **`src/evidence.rs`** - Evidence system (partially duplicated)
5. **`src/schema.rs`** - Pure types and orchestration (149 lines)

## Detailed Complexity Analysis

### 🔴 High Complexity Areas

#### 1. Engine Merging Logic (`src/engine.rs`)

**Current Implementation Analysis:**

```rust
// Lines 40-120: Complex manual merging
fn detect_from_snapshot(&self, snapshot: &EnvSnapshot) -> EnvSense {
    let mut result = EnvSense { /* initialization */ };
    let mut all_contexts = std::collections::HashSet::new();
    let mut all_traits: HashMap<String, serde_json::Value> = HashMap::new();
    let mut all_facets: HashMap<String, serde_json::Value> = HashMap::new();

    // Collect all detections
    for detector in &self.detectors {
        let detection = detector.detect(snapshot);
        // ... collection logic
    }

    // Manual field mapping - 80+ lines of repetitive code
    Self::set_context_bool(&mut result.contexts, "agent", &all_contexts);
    Self::set_context_bool(&mut result.contexts, "ide", &all_contexts);
    Self::set_context_bool(&mut result.contexts, "ci", &all_contexts);
    Self::set_context_bool(&mut result.contexts, "container", &all_contexts);
    Self::set_context_bool(&mut result.contexts, "remote", &all_contexts);

    Self::set_facet_id(&mut result.facets.agent_id, "agent_id", &all_facets);
    Self::set_facet_id(&mut result.facets.ide_id, "ide_id", &all_facets);
    Self::set_facet_id(&mut result.facets.ci_id, "ci_id", &all_facets);

    Self::set_trait_bool(&mut result.traits.is_interactive, "is_interactive", &all_traits);
    Self::set_trait_bool(&mut result.traits.is_tty_stdin, "is_tty_stdin", &all_traits);
    // ... 10+ more trait mappings
}
```

**Complexity Metrics:**

- **Lines of Code**: 80+ lines of repetitive merging logic
- **Cyclomatic Complexity**: High due to multiple conditional branches
- **Maintainability Index**: Low due to repetitive patterns
- **Error Prone**: Manual field mapping with no compile-time validation

**Problems Identified:**

1. **Repetitive Code**: Each field requires a separate `set_*` method call
2. **No Type Safety**: String-based field names prone to typos
3. **Difficult to Extend**: Adding new fields requires updating multiple places
4. **No Validation**: Compile-time errors only discovered at runtime

#### 2. Evidence System Duplication

**Current State Analysis:**

```rust
// src/evidence.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceSource {
    Env, Tty, Proc, Fs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub source: EvidenceSource,
    pub key: String,
    pub value: Option<String>,
    pub supports: Vec<String>,
    pub confidence: f32,
}

// src/schema.rs (duplicate)
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Signal {
    Env, Tty, Proc, Fs,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct Evidence {
    pub signal: Signal,
    pub key: String,
    pub value: Option<String>,
    pub supports: Vec<String>,
    pub confidence: f32,
}
```

**Complexity Metrics:**

- **Code Duplication**: ~30 lines of nearly identical code
- **Naming Inconsistency**: `EvidenceSource` vs `Signal`, `EvidenceItem` vs
  `Evidence`
- **Import Confusion**: Developers must choose between two similar types
- **Maintenance Overhead**: Changes must be made in two places

#### 3. Confidence Scoring Complexity

**Current Implementation Analysis:**

```rust
// src/detectors/ide.rs
detection.confidence = 0.95; // Hardcoded value

// src/detectors/terminal.rs
detection.confidence = 1.0; // Different hardcoded value

// src/detectors/ci.rs
detection.confidence = 0.9; // Yet another hardcoded value
```

**Complexity Metrics:**

- **Magic Numbers**: 3 different hardcoded confidence values
- **No Reasoning**: No documentation for why values differ
- **Inconsistent**: No clear pattern for confidence assignment
- **Difficult to Tune**: Values scattered across multiple files

#### 4. Environment Snapshot Overrides

**Current Implementation Analysis:**

```rust
// src/detectors/mod.rs - Complex override logic
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
```

**Complexity Metrics:**

- **Environment Pollution**: 3 additional environment variables
- **Repetitive Logic**: Same pattern repeated 3 times
- **Error Handling**: Complex fallback logic
- **Testing Complexity**: Must set environment variables for testing

### 🟡 Medium Complexity Areas

#### 5. Detector Registration Pattern

**Current Implementation:**

```rust
// src/schema.rs - Manual detector registration
let engine = DetectionEngine::new()
    .register(TerminalDetector::new())
    .register(AgentDetector::new())
    .register(CiDetector::new())
    .register(IdeDetector::new());
```

**Complexity Metrics:**

- **Boilerplate**: 4 lines of registration code
- **Error Prone**: Easy to forget to register a detector
- **No Validation**: No compile-time check for missing detectors

#### 6. Adapter Pattern in Agent Detector

**Current Implementation:**

```rust
// src/detectors/agent.rs - Adapter pattern
struct EnvSnapshotReader<'a> {
    snapshot: &'a EnvSnapshot,
}

impl<'a> EnvReader for EnvSnapshotReader<'a> {
    fn get(&self, key: &str) -> Option<String> {
        self.snapshot.env_vars.get(key).cloned()
    }
    // ... more adapter methods
}
```

**Complexity Metrics:**

- **Indirection**: Additional layer between EnvSnapshot and agent detection
- **Lifetime Complexity**: Generic lifetime parameter
- **Code Overhead**: ~20 lines of adapter code

### 🟢 Low Complexity Areas

#### 7. Detector Trait Design

**Current Implementation:**

```rust
// src/detectors/mod.rs - Clean trait design
pub trait Detector {
    fn name(&self) -> &'static str;
    fn detect(&self, snap: &EnvSnapshot) -> Detection;
}
```

**Complexity Metrics:**

- **Simple Interface**: 2 methods, clear contract
- **Easy to Implement**: Straightforward trait implementation
- **Good Abstraction**: Hides implementation details

## Complexity Reduction Opportunities

### Immediate Wins (Low Risk)

1. **Evidence Unification**: Eliminate 30 lines of duplicate code
2. **Confidence Constants**: Replace magic numbers with named constants
3. **Builder Pattern**: Add convenience method for detector registration

### Medium-Term Improvements (Medium Risk)

1. **Dependency Injection**: Replace environment overrides with traits
2. **Simplified Confidence**: Use boolean detection with optional confidence
3. **Reduced Adapter Code**: Simplify agent detector integration

### Long-Term Goals (High Risk)

1. **Macro-Based Merging**: Automatic field mapping with derive macros
2. **Schema Evolution**: Improved versioning and compatibility
3. **Performance Optimization**: Reduce allocation and improve efficiency

## Quantitative Analysis

### Code Metrics

| Component             | Lines of Code | Cyclomatic Complexity | Maintainability Index |
| --------------------- | ------------- | --------------------- | --------------------- |
| Engine Merging        | 80+           | High                  | Low                   |
| Evidence Duplication  | 30            | Low                   | Medium                |
| Confidence Scoring    | 10            | Low                   | Low                   |
| Environment Overrides | 15            | Medium                | Medium                |
| Detector Registration | 5             | Low                   | High                  |
| Adapter Pattern       | 20            | Low                   | Medium                |

### Complexity Hotspots

1. **`src/engine.rs`** - 80+ lines of merging logic (highest complexity)
2. **Evidence System** - 30 lines of duplication (highest waste)
3. **Environment Overrides** - 15 lines of repetitive logic (medium complexity)
4. **Adapter Pattern** - 20 lines of indirection (medium overhead)

## Recommendations

### Phase 1: Evidence Unification (Week 1)

- **Impact**: High benefit, low risk
- **Effort**: 1-2 days
- **ROI**: Eliminate 30 lines of duplicate code

### Phase 2: Confidence Simplification (Week 2)

- **Impact**: Medium benefit, low risk
- **Effort**: 1 day
- **ROI**: Improve code clarity and maintainability

### Phase 3: Dependency Injection (Week 3)

- **Impact**: High benefit, medium risk
- **Effort**: 3-4 days
- **ROI**: Improve testability and reduce environment pollution

### Phase 4: Macro-Based Merging (Week 4-5)

- **Impact**: Very high benefit, high risk
- **Effort**: 1-2 weeks
- **ROI**: Eliminate 80+ lines of repetitive code

## Conclusion

The refactoring successfully achieved its architectural goals but introduced
several areas of complexity that can be systematically reduced. The proposed
simplifications would:

1. **Reduce code duplication** by 30+ lines
2. **Eliminate repetitive logic** by 80+ lines
3. **Improve maintainability** through better abstractions
4. **Enhance testability** with dependency injection
5. **Increase type safety** with compile-time validation

The incremental approach ensures we can validate each improvement while
maintaining the successful architectural foundation of the refactoring.
