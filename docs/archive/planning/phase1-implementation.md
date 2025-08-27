# Phase 1 Implementation: Evidence System Unification

This document provides detailed implementation steps for Phase 1 of the simplification proposal - unifying the duplicate evidence types.

## Overview

**Goal**: Eliminate duplicate evidence types and standardize naming across the codebase.

**Current State**: Two nearly identical evidence systems:
- `src/evidence.rs`: `EvidenceItem` and `EvidenceSource`
- `src/schema.rs`: `Evidence` and `Signal`

**Target State**: Single unified evidence system in `src/evidence.rs`

## Implementation Steps

### Step 1: Analyze Current Usage

First, let's understand where each evidence type is used:

```bash
# Find all usages of EvidenceItem and EvidenceSource
grep -r "EvidenceItem\|EvidenceSource" src/

# Find all usages of Evidence and Signal from schema
grep -r "Evidence\|Signal" src/ | grep -v "EvidenceItem\|EvidenceSource"
```

### Step 2: Choose Unified Naming

Based on the analysis, we'll use the schema.rs naming convention:
- `Evidence` (not `EvidenceItem`)
- `Signal` (not `EvidenceSource`)

This choice is based on:
1. `Evidence` is more concise than `EvidenceItem`
2. `Signal` is already used in the public API
3. Schema types are the primary interface

### Step 3: Update src/evidence.rs

Replace the current content with unified types:

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Signal {
    Env,
    Tty,
    Proc,
    Fs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Evidence {
    pub signal: Signal,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default)]
    pub supports: Vec<String>,
    pub confidence: f32,
}

impl Evidence {
    pub fn env_var(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            signal: Signal::Env,
            key: key.into(),
            value: Some(value.into()),
            supports: Vec::new(),
            confidence: 0.9,
        }
    }

    pub fn env_presence(key: impl Into<String>) -> Self {
        Self {
            signal: Signal::Env,
            key: key.into(),
            value: None,
            supports: Vec::new(),
            confidence: 0.9,
        }
    }

    pub fn tty_trait(key: impl Into<String>, is_tty: bool) -> Self {
        Self {
            signal: Signal::Tty,
            key: key.into(),
            value: Some(is_tty.to_string()),
            supports: Vec::new(),
            confidence: 1.0,
        }
    }

    pub fn with_supports(mut self, supports: Vec<String>) -> Self {
        self.supports = supports;
        self
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}
```

### Step 4: Update src/schema.rs

Remove the duplicate evidence definitions and import from evidence module:

```rust
use crate::ci::CiFacet;
use crate::detectors::agent::AgentDetector;
use crate::detectors::ci::CiDetector;
use crate::detectors::ide::IdeDetector;
use crate::detectors::terminal::TerminalDetector;
use crate::engine::DetectionEngine;
use crate::evidence::{Evidence, Signal}; // Import from evidence module
use crate::traits::terminal::{ColorLevel, TerminalTraits};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Remove the duplicate Evidence and Signal definitions
// (lines 12-25 in current schema.rs)

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
pub struct Contexts {
    pub agent: bool,
    pub ide: bool,
    pub ci: bool,
    pub container: bool,
    pub remote: bool,
}

// ... rest of the file remains the same
```

### Step 5: Update Detector Implementations

Update all detector files to use the unified evidence types:

#### src/detectors/ide.rs
```rust
use crate::detectors::{Detection, Detector, EnvSnapshot};
use crate::evidence::{Evidence, Signal}; // Updated import
use serde_json::json;

// ... rest of implementation remains the same
```

#### src/detectors/ci.rs
```rust
use crate::ci::{CiFacet, ci_traits, normalize_vendor};
use crate::detectors::{Detection, Detector, EnvSnapshot};
use crate::evidence::{Evidence, Signal}; // Updated import if needed
use ci_info::types::Vendor;
use serde_json::json;

// ... rest of implementation remains the same
```

#### src/detectors/agent.rs
```rust
use crate::agent::{EnvReader, detect_agent};
use crate::detectors::{Detection, Detector, EnvSnapshot};
use crate::evidence::{Evidence, Signal}; // Updated import
use serde_json::{Value, json};

// ... rest of implementation remains the same
```

### Step 6: Update Tests

Update all test files to use the unified evidence types:

#### tests/cli.rs
```rust
use envsense::schema::EnvSense;
use envsense::evidence::{Evidence, Signal}; // Updated import

// ... rest of tests remain the same
```

#### Unit tests in detector modules
```rust
// In src/detectors/ide.rs tests
use crate::evidence::{Evidence, Signal}; // Updated import

// ... rest of tests remain the same
```

### Step 7: Update Documentation

Update any documentation that references the old evidence types:

#### docs/architecture.md
```markdown
## Evidence Model

Each `Evidence` entry contains:

* `signal` – source of information (`Env`, `Tty`, `Proc`, `Fs`)
* `key` – the specific variable or probe name
* `value` – optional captured value
* `supports` – contexts/facets/traits it backs
* `confidence` – float in `[0.0, 1.0]`

This allows `--explain` to surface reasoning for any true claim.
```

### Step 8: Verify Changes

Run comprehensive tests to ensure everything works:

```bash
# Run all tests
cargo test

# Run baseline validation
./scripts/compare-baseline.sh

# Check for any compilation errors
cargo check

# Verify JSON schema generation
cargo test schema::tests::json_schema_generates
```

### Step 9: Update Imports in lib.rs

Ensure the evidence module is properly exported:

```rust
// src/lib.rs
pub mod agent;
pub mod check;
pub mod ci;
pub mod detectors;
pub mod engine;
pub mod evidence; // Ensure this is exported
pub mod schema;
pub mod traits;

pub use traits::terminal::TerminalTraits;
pub use evidence::{Evidence, Signal}; // Re-export for convenience
```

## Validation Checklist

- [ ] All tests pass (`cargo test`)
- [ ] Baseline validation passes (`./scripts/compare-baseline.sh`)
- [ ] No compilation errors (`cargo check`)
- [ ] JSON schema generation works (`cargo test schema::tests::json_schema_generates`)
- [ ] CLI functionality unchanged (`cargo run -- info --json`)
- [ ] All imports updated to use unified types
- [ ] Documentation updated to reflect unified types
- [ ] No duplicate evidence type definitions remain

## Expected Benefits

1. **Code Reduction**: Eliminate ~30 lines of duplicate code
2. **Consistency**: Single source of truth for evidence types
3. **Maintainability**: Changes only need to be made in one place
4. **Clarity**: Clear naming convention throughout codebase

## Rollback Plan

If issues arise during implementation:

1. **Git Branch**: Work on a feature branch for easy rollback
2. **Incremental Commits**: Commit each step separately
3. **Test After Each Step**: Verify functionality before proceeding
4. **Documentation**: Keep notes of all changes made

## Next Steps

After successful completion of Phase 1:

1. **Document Results**: Update this document with actual outcomes
2. **Measure Impact**: Count lines of code eliminated
3. **Plan Phase 2**: Begin confidence scoring simplification
4. **Update Roadmap**: Adjust timeline based on Phase 1 experience

## Troubleshooting

### Common Issues

1. **Import Errors**: Ensure all files import from `crate::evidence`
2. **Serialization Issues**: Verify `serde` attributes are consistent
3. **Test Failures**: Check that test data matches new type structure
4. **Schema Generation**: Ensure `JsonSchema` derive is present

### Debugging Commands

```bash
# Check for unused imports
cargo clippy -- -D warnings

# Check for dead code
cargo clippy -- -D dead_code

# Verify no duplicate type definitions
grep -r "struct Evidence\|enum Signal" src/

# Check JSON serialization
cargo run -- info --json | jq '.evidence[0]'
```

This implementation guide provides a step-by-step approach to safely unify the evidence system while maintaining functionality and improving code quality.
