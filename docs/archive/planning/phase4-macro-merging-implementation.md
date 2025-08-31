# Phase 4: Macro-Based Engine Merging Implementation Plan

## Overview

This document outlines the detailed implementation plan for Phase 4 of the
simplification proposal: replacing the manual engine merging logic with derive
macros for automatic field mapping. This is the highest-risk phase but offers
the greatest potential for complexity reduction.

## Current State Analysis

### Current Engine Merging Logic (`src/engine.rs` lines 40-120)

The current implementation has several complexity issues:

1. **Manual Field Mapping**: 80+ lines of repetitive merging logic
2. **Hardcoded Field Names**: String-based field matching prone to typos
3. **No Compile-Time Validation**: Field mappings not validated at compile time
4. **Difficult Maintenance**: Adding new fields requires manual updates in
   multiple places
5. **Mixed Logic**: Simple boolean assignments mixed with complex enum/struct
   parsing

### Current Structure

```rust
// Current manual merging approach
Self::set_context_bool(&mut result.contexts, "agent", &all_contexts);
Self::set_facet_id(&mut result.facets.agent_id, "agent_id", &all_facets);
Self::set_trait_bool(&mut result.traits.is_interactive, "is_interactive", &all_traits);

// Plus special cases for complex types
if let Some(color_level_str) = all_traits.get("color_level").and_then(|v| v.as_str()) {
    result.traits.color_level = match color_level_str {
        "none" => ColorLevel::None,
        "ansi16" => ColorLevel::Ansi16,
        // ... more cases
    };
}
```

## Proposed Solution: Derive Macro Approach

### Core Concept

Create a custom derive macro `#[derive(DetectionMerger)]` that automatically
generates merging logic based on field annotations and type information.

### Macro Design

#### 1. Field Annotations

```rust
#[derive(DetectionMerger)]
pub struct EnvSense {
    #[detection_merge(contexts = "agent")]
    pub contexts: Contexts,

    #[detection_merge(facets = "agent_id")]
    pub facets: Facets,

    #[detection_merge(traits = "is_interactive")]
    pub traits: Traits,

    #[detection_merge(evidence)]
    pub evidence: Vec<Evidence>,

    pub version: String,
    pub rules_version: String,
}
```

#### 2. Generated Implementation

The macro would generate:

```rust
impl DetectionMerger for EnvSense {
    fn merge_detections(&mut self, detections: &[Detection]) {
        let mut all_contexts = std::collections::HashSet::new();
        let mut all_traits: HashMap<String, serde_json::Value> = HashMap::new();
        let mut all_facets: HashMap<String, serde_json::Value> = HashMap::new();

        // Collect all detection data
        for detection in detections {
            all_contexts.extend(detection.contexts_add.iter().cloned());
            all_traits.extend(detection.traits_patch.clone());
            all_facets.extend(detection.facets_patch.clone());
            self.evidence.extend(detection.evidence.clone());
        }

        // Auto-generated merging logic
        self.contexts.agent = all_contexts.contains("agent");
        self.contexts.ide = all_contexts.contains("ide");
        // ... other contexts

        if let Some(value) = all_facets.get("agent_id").and_then(|v| v.as_str()) {
            self.facets.agent_id = Some(value.to_string());
        }
        // ... other facets

        if let Some(value) = all_traits.get("is_interactive").and_then(|v| v.as_bool()) {
            self.traits.is_interactive = value;
        }
        // ... other traits

        // Special handling for complex types
        self.merge_color_level(&all_traits);
        self.merge_ci_facet(&all_facets);
    }
}
```

## Implementation Plan

### Why Separate Macro Crate?

The decision to create a separate `envsense-macros` crate follows Rust ecosystem
best practices:

1. **Ecosystem Standard**: Most major Rust projects use separate macro crates
   (serde_derive, tokio-macros, etc.)
2. **Clean Separation**: Compile-time macro code is separate from runtime
   application code
3. **Dependency Management**: Macro dependencies (proc-macro2, syn, quote) are
   different from runtime dependencies
4. **Reusability**: Other projects can use the macros without pulling in the
   main crate
5. **Build Optimization**: Macros don't need to be included in the final binary

### Repository Structure Conventions

Following Rust ecosystem conventions for multi-crate repositories:

**Standard Layout:**

- **Main crate**: `src/` directory at repository root (current structure)
- **Macro crate**: `project-name-macros/` subdirectory
- **Workspace root**: `Cargo.toml` at repository root defines workspace members

**Examples from Major Projects:**

- **Serde**: `serde/` (main) + `serde_derive/` (macros)
- **Tokio**: `tokio/` (main) + `tokio-macros/` (macros)
- **Thiserror**: `thiserror/` (main) + `thiserror-impl/` (macros)

**Benefits:**

- **Single repository**: All related code in one place
- **Shared CI/CD**: Common testing and deployment pipeline
- **Version coordination**: Easy to keep crates in sync
- **Documentation**: Single source of truth for all components

### Phase 4A: Macro Infrastructure (Week 1)

#### Step 1: Create Macro Crate Structure

```bash
# Create new crate for macros within the workspace
mkdir envsense-macros
cd envsense-macros
cargo init --lib
```

**Workspace Setup:**

```bash
# Update root Cargo.toml to add workspace configuration
# Add [workspace] section with members = [".", "envsense-macros"]
```

**File Structure:**

```
envsense/
├── Cargo.toml              # Workspace root
├── src/                    # Main crate source (existing)
├── envsense-macros/        # Macro crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── detection_merger.rs
│   │   └── utils.rs
│   └── tests/
│       └── detection_merger_tests.rs
├── tests/                  # Main crate integration tests (existing)
└── docs/                   # Documentation (existing)
```

#### Step 2: Define Macro Trait

```rust
// envsense-macros/src/lib.rs
pub trait DetectionMerger {
    fn merge_detections(&mut self, detections: &[crate::detectors::Detection]);
}
```

#### Step 3: Basic Macro Implementation

Start with a simple macro that handles basic field mapping:

```rust
// envsense-macros/src/detection_merger.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(DetectionMerger)]
pub fn derive_detection_merger(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Parse struct fields and generate merging logic
    let struct_name = input.ident;
    let fields = parse_fields(&input.data);

    let merge_impl = generate_merge_impl(&struct_name, &fields);

    TokenStream::from(quote! {
        impl DetectionMerger for #struct_name {
            #merge_impl
        }
    })
}
```

### Phase 4B: Field Parsing and Mapping (Week 2)

#### Step 1: Parse Field Annotations

```rust
#[derive(Debug)]
struct FieldMapping {
    field_name: String,
    mapping_type: MappingType,
    target_key: Option<String>,
}

#[derive(Debug)]
enum MappingType {
    Contexts,
    Facets,
    Traits,
    Evidence,
    Ignore,
}

fn parse_field_annotations(field: &syn::Field) -> Option<FieldMapping> {
    // Parse #[detection_merge(...)] attributes
    // Extract mapping type and target key
}
```

#### Step 2: Generate Basic Merging Logic

```rust
fn generate_contexts_merging(fields: &[FieldMapping]) -> proc_macro2::TokenStream {
    // Generate code for boolean context merging
    quote! {
        // Set boolean contexts
        #(self.#field_name = all_contexts.contains(#target_key);)*
    }
}
```

#### Step 3: Handle Different Field Types

```rust
fn generate_field_merging(field: &FieldMapping) -> proc_macro2::TokenStream {
    match field.mapping_type {
        MappingType::Contexts => generate_contexts_merging(field),
        MappingType::Facets => generate_facets_merging(field),
        MappingType::Traits => generate_traits_merging(field),
        MappingType::Evidence => generate_evidence_merging(field),
        MappingType::Ignore => quote! {},
    }
}
```

### Phase 4C: Complex Type Handling (Week 3)

#### Step 1: Enum Type Support

Handle complex types like `ColorLevel`:

```rust
fn generate_enum_merging(field: &FieldMapping, enum_type: &str) -> proc_macro2::TokenStream {
    match enum_type {
        "ColorLevel" => quote! {
            if let Some(color_level_str) = all_traits.get(#target_key).and_then(|v| v.as_str()) {
                self.#field_name = match color_level_str {
                    "none" => ColorLevel::None,
                    "ansi16" => ColorLevel::Ansi16,
                    "ansi256" => ColorLevel::Ansi256,
                    "truecolor" => ColorLevel::Truecolor,
                    _ => ColorLevel::None,
                };
            }
        },
        // Add other enum types as needed
    }
}
```

#### Step 2: Struct Type Support

Handle complex struct types like `CiFacet`:

```rust
fn generate_struct_merging(field: &FieldMapping, struct_type: &str) -> proc_macro2::TokenStream {
    match struct_type {
        "CiFacet" => quote! {
            if let Some(ci_facet_value) = all_facets.get(#target_key)
                && let Ok(ci_facet) = serde_json::from_value::<CiFacet>(ci_facet_value.clone())
            {
                self.#field_name = ci_facet;
            }
        },
        // Add other struct types as needed
    }
}
```

#### Step 3: Type Detection

```rust
fn detect_field_type(field: &syn::Field) -> FieldType {
    // Analyze field type to determine if it's:
    // - Basic type (bool, String, Option<String>)
    // - Enum type (ColorLevel)
    // - Struct type (CiFacet)
    // - Collection type (Vec<Evidence>)
}
```

### Phase 4D: Integration and Testing (Week 4)

#### Step 1: Update Main Crate

```rust
// src/engine.rs
use envsense_macros::DetectionMerger;

#[derive(DetectionMerger)]
pub struct EnvSense {
    #[detection_merge(contexts = "agent")]
    pub contexts: Contexts,

    #[detection_merge(facets = "agent_id")]
    pub facets: Facets,

    #[detection_merge(traits = "is_interactive")]
    pub traits: Traits,

    #[detection_merge(evidence)]
    pub evidence: Vec<Evidence>,

    pub version: String,
    pub rules_version: String,
}

impl DetectionEngine {
    pub fn detect_from_snapshot(&self, snapshot: &EnvSnapshot) -> EnvSense {
        let mut result = EnvSense::default();
        let detections: Vec<Detection> = self.detectors
            .iter()
            .map(|detector| detector.detect(snapshot))
            .collect();

        result.merge_detections(&detections);
        result
    }
}
```

#### Step 2: Comprehensive Testing

```rust
// tests/macro_integration_tests.rs
#[test]
fn test_macro_generated_merging() {
    let engine = DetectionEngine::new()
        .register(TerminalDetector::new())
        .register(AgentDetector::new());

    let result = engine.detect();

    // Verify all fields are properly merged
    assert!(result.contexts.agent || !result.contexts.agent); // Boolean logic
    assert!(result.traits.is_interactive || !result.traits.is_interactive);
    // ... more assertions
}

#[test]
fn test_macro_with_mock_detections() {
    let mut envsense = EnvSense::default();
    let detections = vec![
        Detection {
            contexts_add: vec!["agent".to_string()],
            traits_patch: HashMap::from([
                ("is_interactive".to_string(), serde_json::Value::Bool(true))
            ]),
            facets_patch: HashMap::from([
                ("agent_id".to_string(), serde_json::Value::String("test-agent".to_string()))
            ]),
            evidence: vec![],
            confidence: 1.0,
        }
    ];

    envsense.merge_detections(&detections);

    assert!(envsense.contexts.agent);
    assert!(envsense.traits.is_interactive);
    assert_eq!(envsense.facets.agent_id, Some("test-agent".to_string()));
}
```

## Alternative Approaches

### Option A: Attribute-Based Configuration

```rust
#[derive(DetectionMerger)]
#[detection_merge(
    contexts_field = "contexts",
    facets_field = "facets",
    traits_field = "traits",
    evidence_field = "evidence"
)]
pub struct EnvSense {
    pub contexts: Contexts,
    pub facets: Facets,
    pub traits: Traits,
    pub evidence: Vec<Evidence>,
    pub version: String,
    pub rules_version: String,
}
```

### Option B: Convention-Based Mapping

```rust
#[derive(DetectionMerger)]
pub struct EnvSense {
    // Automatically map based on field names and types
    pub contexts: Contexts,  // Maps to contexts_add
    pub facets: Facets,      // Maps to facets_patch
    pub traits: Traits,      // Maps to traits_patch
    pub evidence: Vec<Evidence>, // Maps to evidence
    pub version: String,     // Ignored (no mapping)
    pub rules_version: String, // Ignored (no mapping)
}
```

### Option C: Builder Pattern with Macros

```rust
#[derive(DetectionMergerBuilder)]
pub struct EnvSense {
    pub contexts: Contexts,
    pub facets: Facets,
    pub traits: Traits,
    pub evidence: Vec<Evidence>,
    pub version: String,
    pub rules_version: String,
}

// Usage
let mut envsense = EnvSense::default();
envsense
    .merge_contexts(&detection.contexts_add)
    .merge_facets(&detection.facets_patch)
    .merge_traits(&detection.traits_patch)
    .merge_evidence(&detection.evidence);
```

## Risk Mitigation

### 1. Incremental Implementation

- Start with simple boolean field mapping
- Add complex type support gradually
- Maintain backward compatibility throughout

### 2. Comprehensive Testing

- Unit tests for each macro component
- Integration tests with real detector data
- Property-based testing for edge cases
- Performance benchmarking

### 3. Fallback Strategy

- Keep existing manual merging logic as backup
- Feature flag to switch between macro and manual approaches
- Gradual migration with validation

### 4. Documentation and Examples

- Comprehensive macro documentation
- Example usage patterns
- Migration guide from manual to macro approach

## Success Metrics

### Code Reduction

- **Target**: 80+ lines of manual merging logic → ~20 lines of macro annotations
- **Measurement**: Line count comparison between current and macro-based
  implementations

### Maintainability

- **Target**: Adding new fields requires only struct definition changes
- **Measurement**: Time to add new detector fields

### Performance

- **Target**: No performance regression
- **Measurement**: Benchmark comparison with current implementation

### Compile-Time Safety

- **Target**: Compile-time validation of field mappings
- **Measurement**: Compile errors for invalid field mappings

## Implementation Timeline

### Week 1: Macro Infrastructure

- [ ] Create `envsense-macros` crate
- [ ] Define `DetectionMerger` trait
- [ ] Basic macro structure and parsing

### Week 2: Field Mapping

- [ ] Parse field annotations
- [ ] Generate basic merging logic
- [ ] Handle different field types

### Week 3: Complex Types

- [ ] Enum type support (ColorLevel)
- [ ] Struct type support (CiFacet)
- [ ] Type detection and validation

### Week 4: Integration

- [ ] Update main crate to use macro
- [ ] Comprehensive testing
- [ ] Performance validation
- [ ] Documentation

## Dependencies

### New Dependencies

The macro crate will have its own dependencies separate from the main crate:

```toml
# envsense-macros/Cargo.toml
[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
```

**Note**: These are compile-time dependencies that won't be included in the
final binary.

### Updated Dependencies

#### Workspace Configuration

```toml
# Cargo.toml (root - workspace configuration)
[workspace]
members = [
    ".",                    # Main crate (current directory)
    "envsense-macros"       # Macro crate
]

# Main crate configuration (existing)
[package]
name = "envsense"
version = "0.1.0"
# ... rest of existing config

[dependencies]
envsense-macros = { path = "./envsense-macros" }
```

#### Macro Crate Dependencies

```toml
# envsense-macros/Cargo.toml
[package]
name = "envsense-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
```

## Conclusion

Phase 4 represents the highest-risk but highest-reward simplification. The
macro-based approach has the potential to:

1. **Reduce complexity** by 60-80% in the engine merging logic
2. **Improve maintainability** through automatic field mapping
3. **Enhance safety** with compile-time validation
4. **Simplify extension** for new detector fields

The incremental implementation approach with comprehensive testing and fallback
strategies minimizes risk while maximizing the potential benefits.

## Next Steps

1. **Review and approve** this implementation plan
2. **Set up macro infrastructure** (Week 1)
3. **Implement basic field mapping** (Week 2)
4. **Add complex type support** (Week 3)
5. **Integrate and test** (Week 4)
6. **Evaluate results** and decide on full adoption
