# Simplification Proposal for EnvSense Refactoring

This document proposes specific simplifications to reduce complexity in the refactored EnvSense codebase while maintaining functionality and improving maintainability.

## Executive Summary

The refactoring successfully achieved its goals but introduced some complexity that can be reduced. This proposal focuses on:

1. **Engine Merging Logic** - Replace manual field mapping with derive macros
2. **Evidence System Unification** - Merge duplicate evidence types
3. **Confidence Scoring Simplification** - Reduce hardcoded confidence values
4. **Environment Snapshot Overrides** - Use dependency injection for testability
5. **Detector Registration** - Simplify the builder pattern

## Current Complexity Analysis

### 🔴 High Complexity Areas

#### 1. Engine Merging Logic (`src/engine.rs` lines 40-120)

**Current Implementation:**
```rust
// Complex manual merging with many set_* methods
Self::set_context_bool(&mut result.contexts, "agent", &all_contexts);
Self::set_facet_id(&mut result.facets.agent_id, "agent_id", &all_facets);
Self::set_trait_bool(&mut result.traits.is_interactive, "is_interactive", &all_traits);
```

**Problems:**
- 80+ lines of repetitive merging logic
- Manual field mapping prone to errors
- Difficult to maintain when adding new fields
- No compile-time validation of field mappings

#### 2. Evidence System Duplication

**Current State:**
- `src/evidence.rs` defines `EvidenceItem` and `EvidenceSource`
- `src/schema.rs` defines `Evidence` and `Signal`
- Nearly identical structures with different names

**Problems:**
- Code duplication
- Confusing naming
- Potential for inconsistency

#### 3. Hardcoded Confidence Values

**Current Implementation:**
```rust
detection.confidence = 0.95; // Most detectors use hardcoded values
detection.confidence = 1.0;  // Terminal detection
```

**Problems:**
- Arbitrary confidence values
- No clear reasoning for different confidence levels
- Difficult to tune or understand

#### 4. Environment Snapshot Overrides

**Current Implementation:**
```rust
let is_tty_stdin = env_vars
    .get("ENVSENSE_TTY_STDIN")
    .and_then(|v| v.parse::<bool>().ok())
    .unwrap_or_else(|| std::io::stdin().is_terminal());
```

**Problems:**
- Complex override logic
- Environment variable pollution
- Difficult to test individual components

## Proposed Simplifications

### 🟢 Phase 1: Evidence System Unification (Low Risk)

**Goal:** Eliminate duplicate evidence types and standardize naming.

**Implementation:**
```rust
// src/evidence.rs - Single unified evidence system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub signal: Signal,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default)]
    pub supports: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Signal {
    Env,
    Tty,
    Proc,
    Fs,
}

// Remove duplicate definitions from schema.rs
```

**Benefits:**
- Eliminates code duplication
- Clearer naming convention
- Single source of truth for evidence

**Migration:**
1. Update `src/evidence.rs` with unified types
2. Remove duplicate definitions from `src/schema.rs`
3. Update all imports and references
4. Update tests to use unified types

### 🟡 Phase 2: Confidence Scoring Simplification (Medium Risk)

**Goal:** Simplify confidence scoring with clear, meaningful values.

**Implementation:**
```rust
// src/detectors/mod.rs - Add confidence constants
pub const CONFIDENCE_HIGH: f32 = 1.0;    // Direct env var match
pub const CONFIDENCE_MEDIUM: f32 = 0.8;  // Inferred from context
pub const CONFIDENCE_LOW: f32 = 0.6;     // Heuristic detection

// Usage in detectors
impl Detector for IdeDetector {
    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();
        
        if let Some(term_program) = snap.get_env("TERM_PROGRAM")
            && term_program == "vscode"
        {
            detection.contexts_add.push("ide".to_string());
            detection.confidence = CONFIDENCE_HIGH; // Clear reasoning
            // ... rest of detection logic
        }
        
        detection
    }
}
```

**Alternative: Boolean Detection with Optional Confidence**
```rust
pub struct Detection {
    pub contexts_add: Vec<String>,
    pub traits_patch: HashMap<String, serde_json::Value>,
    pub facets_patch: HashMap<String, serde_json::Value>,
    pub evidence: Vec<Evidence>,
    pub confidence: Option<f32>, // Optional for simple cases
}

// Default to None for simple boolean detection
impl Default for Detection {
    fn default() -> Self {
        Self {
            contexts_add: Vec::new(),
            traits_patch: HashMap::new(),
            facets_patch: HashMap::new(),
            evidence: Vec::new(),
            confidence: None, // No confidence by default
        }
    }
}
```

**Benefits:**
- Clear reasoning for confidence values
- Optional confidence for simple cases
- Easier to understand and maintain

### 🟠 Phase 3: Dependency Injection for Testability (Medium Risk)

**Goal:** Replace environment variable overrides with proper dependency injection.

**Implementation:**
```rust
// src/detectors/mod.rs - Add trait for TTY detection
pub trait TtyDetector {
    fn is_tty_stdin(&self) -> bool;
    fn is_tty_stdout(&self) -> bool;
    fn is_tty_stderr(&self) -> bool;
}

// Real implementation
pub struct RealTtyDetector;

impl TtyDetector for RealTtyDetector {
    fn is_tty_stdin(&self) -> bool {
        std::io::stdin().is_terminal()
    }
    
    fn is_tty_stdout(&self) -> bool {
        std::io::stdout().is_terminal()
    }
    
    fn is_tty_stderr(&self) -> bool {
        std::io::stderr().is_terminal()
    }
}

// Mock implementation for testing
pub struct MockTtyDetector {
    pub stdin: bool,
    pub stdout: bool,
    pub stderr: bool,
}

impl TtyDetector for MockTtyDetector {
    fn is_tty_stdin(&self) -> bool { self.stdin }
    fn is_tty_stdout(&self) -> bool { self.stdout }
    fn is_tty_stderr(&self) -> bool { self.stderr }
}

// Update EnvSnapshot to use dependency injection
pub struct EnvSnapshot {
    pub env_vars: HashMap<String, String>,
    pub tty_detector: Box<dyn TtyDetector>,
}

impl EnvSnapshot {
    pub fn current() -> Self {
        Self {
            env_vars: std::env::vars().collect(),
            tty_detector: Box::new(RealTtyDetector),
        }
    }
    
    pub fn for_testing(env_vars: HashMap<String, String>, tty_detector: Box<dyn TtyDetector>) -> Self {
        Self { env_vars, tty_detector }
    }
    
    pub fn is_tty_stdin(&self) -> bool {
        self.tty_detector.is_tty_stdin()
    }
    
    pub fn is_tty_stdout(&self) -> bool {
        self.tty_detector.is_tty_stdout()
    }
    
    pub fn is_tty_stderr(&self) -> bool {
        self.tty_detector.is_tty_stderr()
    }
}
```

**Benefits:**
- Cleaner testability
- No environment variable pollution
- Clear separation of concerns
- Easier to mock for testing

### 🔴 Phase 4: Macro-Based Engine Merging (High Risk)

**Goal:** Replace manual merging logic with derive macros for automatic field mapping.

**Implementation:**
```rust
// Create a custom derive macro for automatic merging
#[derive(DetectionMerger)]
pub struct EnvSense {
    pub contexts: Contexts,
    pub facets: Facets,
    pub traits: Traits,
    pub evidence: Vec<Evidence>,
    pub version: String,
    pub rules_version: String,
}

// The macro would generate something like:
impl DetectionMerger for EnvSense {
    fn merge_detections(&mut self, detections: &[Detection]) {
        for detection in detections {
            // Auto-map contexts
            for context in &detection.contexts_add {
                match context.as_str() {
                    "agent" => self.contexts.agent = true,
                    "ide" => self.contexts.ide = true,
                    "ci" => self.contexts.ci = true,
                    "container" => self.contexts.container = true,
                    "remote" => self.contexts.remote = true,
                    _ => {} // Unknown context
                }
            }
            
            // Auto-map facets
            for (key, value) in &detection.facets_patch {
                match key.as_str() {
                    "agent_id" => self.facets.agent_id = value.as_str().map(|s| s.to_string()),
                    "ide_id" => self.facets.ide_id = value.as_str().map(|s| s.to_string()),
                    "ci_id" => self.facets.ci_id = value.as_str().map(|s| s.to_string()),
                    "container_id" => self.facets.container_id = value.as_str().map(|s| s.to_string()),
                    _ => {} // Unknown facet
                }
            }
            
            // Auto-map traits
            for (key, value) in &detection.traits_patch {
                match key.as_str() {
                    "is_interactive" => self.traits.is_interactive = value.as_bool().unwrap_or(false),
                    "is_tty_stdin" => self.traits.is_tty_stdin = value.as_bool().unwrap_or(false),
                    "is_tty_stdout" => self.traits.is_tty_stdout = value.as_bool().unwrap_or(false),
                    "is_tty_stderr" => self.traits.is_tty_stderr = value.as_bool().unwrap_or(false),
                    "is_piped_stdin" => self.traits.is_piped_stdin = value.as_bool().unwrap_or(false),
                    "is_piped_stdout" => self.traits.is_piped_stdout = value.as_bool().unwrap_or(false),
                    "supports_hyperlinks" => self.traits.supports_hyperlinks = value.as_bool().unwrap_or(false),
                    _ => {} // Unknown trait
                }
            }
            
            // Merge evidence
            self.evidence.extend(detection.evidence.clone());
        }
    }
}
```

**Alternative: Builder Pattern Simplification**
```rust
// Simplify detector registration
impl DetectionEngine {
    pub fn with_all_detectors() -> Self {
        Self::new()
            .register(TerminalDetector::new())
            .register(AgentDetector::new())
            .register(CiDetector::new())
            .register(IdeDetector::new())
    }
}

// Usage becomes simpler
let engine = DetectionEngine::with_all_detectors();
let result = engine.detect();
```

**Benefits:**
- Automatic field mapping
- Compile-time validation
- Reduced boilerplate code
- Easier to add new fields

## Implementation Plan

### Phase 1: Evidence Unification (Week 1)
- [ ] Create unified evidence types in `src/evidence.rs`
- [ ] Update all detector implementations
- [ ] Update tests and snapshots
- [ ] Remove duplicate definitions

### Phase 2: Confidence Simplification (Week 2)
- [ ] Add confidence constants
- [ ] Update detector implementations
- [ ] Consider boolean detection approach
- [ ] Update tests

### Phase 3: Dependency Injection (Week 3)
- [ ] Create TtyDetector trait
- [ ] Implement real and mock detectors
- [ ] Update EnvSnapshot to use dependency injection
- [ ] Update tests to use mock detectors

### Phase 4: Macro-Based Merging (Week 4-5)
- [ ] Research derive macro implementation
- [ ] Create DetectionMerger derive macro
- [ ] Replace manual merging logic
- [ ] Comprehensive testing

## Risk Assessment

### Low Risk
- **Evidence Unification**: Simple refactoring with clear benefits
- **Confidence Simplification**: Well-contained changes

### Medium Risk
- **Dependency Injection**: Requires careful testing but improves architecture
- **Builder Pattern Simplification**: Safe but limited impact

### High Risk
- **Macro-Based Merging**: Complex implementation, requires thorough testing
- **Schema Changes**: May affect external consumers

## Success Metrics

1. **Code Reduction**: Target 30-50% reduction in engine merging logic
2. **Test Coverage**: Maintain 100% test coverage throughout changes
3. **Performance**: No regression in detection performance
4. **Maintainability**: Easier to add new detectors and fields
5. **Documentation**: Updated documentation reflecting simplified architecture

## Alternative Approaches

### Option A: Incremental Simplification
- Focus on low-risk changes first
- Evaluate impact before proceeding to high-risk changes
- Maintain current functionality throughout

### Option B: Minimal Changes
- Only implement evidence unification and confidence simplification
- Keep current engine merging logic
- Focus on documentation and testing improvements

### Option C: Full Simplification
- Implement all proposed changes
- Comprehensive refactoring with significant complexity reduction
- Higher risk but maximum benefit

## Recommendation

**Recommended Approach: Option A (Incremental Simplification)**

1. Start with Phase 1 (Evidence Unification) - low risk, high benefit
2. Proceed with Phase 2 (Confidence Simplification) - medium risk, clear benefit
3. Evaluate Phase 3 (Dependency Injection) based on Phase 1-2 results
4. Consider Phase 4 (Macro-Based Merging) only if significant complexity remains

This approach provides immediate benefits while minimizing risk and allowing for course correction based on early results.

## Conclusion

The proposed simplifications would significantly reduce complexity while maintaining or improving functionality. The incremental approach ensures we can validate each change before proceeding to more complex modifications.

The refactoring was successful, but these simplifications would make the codebase more maintainable and easier to extend in the future.
