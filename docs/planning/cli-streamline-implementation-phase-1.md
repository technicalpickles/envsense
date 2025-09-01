# CLI Streamlining Implementation - Phase 1: Foundation & Schema

## Overview

Phase 1 establishes the foundational changes required for the CLI streamlining
effort. This phase focuses on creating the new nested schema structure while
maintaining backward compatibility, ensuring a smooth transition for existing
users.

## Objective

Establish the new nested schema structure while maintaining backward
compatibility with the existing 0.2.0 schema.

## Timeline

**Estimated Duration**: 2-3 weeks  
**Dependencies**: None (foundation phase)  
**Risk Level**: Medium (schema changes affect all consumers)

## Detailed Tasks

### 1.1 Create New Trait Structures

**Files**: `src/schema.rs`, `src/traits/`

**Objective**: Define the new nested trait structures that will replace the flat
`Facets` and `Traits` system.

#### Implementation Details

```rust
// New nested trait structures
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct AgentTraits {
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct IdeTraits {
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct TerminalTraits {
    pub interactive: bool,
    pub color_level: ColorLevel,
    pub stdin: StreamInfo,
    pub stdout: StreamInfo,
    pub stderr: StreamInfo,
    pub supports_hyperlinks: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct StreamInfo {
    pub tty: bool,
    pub piped: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct CiTraits {
    pub id: Option<String>,
    pub vendor: Option<String>,
    pub name: Option<String>,
    pub is_pr: Option<bool>,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct NestedTraits {
    pub agent: AgentTraits,
    pub ide: IdeTraits,
    pub terminal: TerminalTraits,
    pub ci: CiTraits,
}
```

#### Key Design Decisions

1. **Optional Fields**: Most fields are `Option<T>` to handle cases where
   detection fails
2. **Nested Structure**: Each context (agent, ide, terminal, ci) has its own
   trait struct
3. **Stream Information**: Terminal traits include detailed stream information
   for better debugging
4. **Boolean Flags**: Interactive and hyperlink support are boolean for simple
   evaluation

#### Implementation Steps

1. Create new trait structs in `src/traits/mod.rs`
2. Add `Default` implementations for all structs
3. Ensure all structs implement required derive macros
4. Add comprehensive documentation comments

### 1.2 Update Main Schema

**Files**: `src/schema.rs`

**Objective**: Update the main `EnvSense` struct to use the new nested schema
while maintaining backward compatibility.

#### Implementation Details

```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, DetectionMergerDerive)]
pub struct EnvSense {
    pub contexts: Vec<String>,  // Simplified from Contexts struct
    pub traits: NestedTraits,   // New nested structure
    pub evidence: Vec<Evidence>,
    pub version: String,
}

// Maintain backward compatibility
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq)]
pub struct LegacyEnvSense {
    pub contexts: Contexts,
    pub facets: Facets,
    pub traits: Traits,
    pub evidence: Vec<Evidence>,
    pub version: String,
}
```

#### Schema Changes

1. **Contexts Simplification**: Change from `Contexts` struct to `Vec<String>`
   for simpler handling
2. **Traits Restructuring**: Replace flat `Traits` with nested `NestedTraits`
3. **Facets Removal**: Remove `Facets` field (functionality moved to nested
   traits)
4. **Version Bump**: Increment to 0.3.0

#### Backward Compatibility

- Maintain `LegacyEnvSense` struct for existing consumers
- Provide conversion functions between old and new schemas
- Ensure JSON serialization maintains expected structure for legacy consumers

### 1.3 Schema Version Management

**Files**: `src/schema.rs`

**Objective**: Implement version management and conversion functions between old
and new schemas.

#### Implementation Details

```rust
pub const SCHEMA_VERSION: &str = "0.3.0";
pub const LEGACY_SCHEMA_VERSION: &str = "0.2.0";

impl EnvSense {
    pub fn to_legacy(&self) -> LegacyEnvSense {
        // Convert new schema to legacy format
        LegacyEnvSense {
            contexts: Contexts {
                agent: self.contexts.contains(&"agent".to_string()),
                ide: self.contexts.contains(&"ide".to_string()),
                ci: self.contexts.contains(&"ci".to_string()),
            },
            facets: Facets {
                agent_id: self.traits.agent.id.clone(),
                ide_id: self.traits.ide.id.clone(),
                // Map other fields as needed
            },
            traits: Traits {
                is_interactive: self.traits.terminal.interactive,
                color_level: self.traits.terminal.color_level.clone(),
                // Map other traits
            },
            evidence: self.evidence.clone(),
            version: LEGACY_SCHEMA_VERSION.to_string(),
        }
    }

    pub fn from_legacy(legacy: &LegacyEnvSense) -> Self {
        // Convert legacy schema to new format
        let mut contexts = Vec::new();
        if legacy.contexts.agent { contexts.push("agent".to_string()); }
        if legacy.contexts.ide { contexts.push("ide".to_string()); }
        if legacy.contexts.ci { contexts.push("ci".to_string()); }

        Self {
            contexts,
            traits: NestedTraits {
                agent: AgentTraits {
                    id: legacy.facets.agent_id.clone(),
                },
                ide: IdeTraits {
                    id: legacy.facets.ide_id.clone(),
                },
                terminal: TerminalTraits {
                    interactive: legacy.traits.is_interactive,
                    color_level: legacy.traits.color_level.clone(),
                    stdin: StreamInfo { tty: false, piped: false }, // Default values
                    stdout: StreamInfo { tty: false, piped: false },
                    stderr: StreamInfo { tty: false, piped: false },
                    supports_hyperlinks: false,
                },
                ci: CiTraits {
                    id: None, // Legacy doesn't have CI info
                    vendor: None,
                    name: None,
                    is_pr: None,
                    branch: None,
                },
            },
            evidence: legacy.evidence.clone(),
            version: SCHEMA_VERSION.to_string(),
        }
    }
}
```

#### Conversion Logic

1. **Context Mapping**: Convert boolean context flags to string list
2. **Field Mapping**: Map legacy facet fields to appropriate nested trait
   locations
3. **Default Values**: Provide sensible defaults for new fields not present in
   legacy schema
4. **Version Handling**: Ensure proper version strings in converted schemas

### 1.4 Update Macro System

**Files**: `envsense-macros/src/`, `envsense-macros-impl/src/`

**Objective**: Extend the `DetectionMergerDerive` macro to handle nested
structures while maintaining backward compatibility.

#### Implementation Details

```rust
// Extend macro to handle nested object merging
fn generate_nested_merge_impl(input: &DeriveInput) -> TokenStream {
    // Generate merging logic for nested structures
    // Handle cases where traits_patch contains nested objects
    // Merge them into the appropriate nested trait fields
}
```

#### Macro Changes Required

1. **Nested Object Support**: Handle merging of nested JSON objects into trait
   structs
2. **Field Path Resolution**: Support merging at different nesting levels
3. **Type Safety**: Ensure merged objects match expected trait types
4. **Backward Compatibility**: Maintain support for flat trait merging

#### Implementation Steps

1. Analyze existing macro implementation
2. Extend merge logic to handle nested structures
3. Add tests for nested merging scenarios
4. Ensure performance doesn't degrade with nested structures

### 1.5 Tests & Validation

**Files**: `tests/`, `src/schema.rs`

**Objective**: Comprehensive testing of new schema structures, serialization,
and conversion functions.

#### Test Coverage Requirements

```rust
#[test]
fn new_schema_serialization() {
    let env = EnvSense::default();
    let json = serde_json::to_string(&env).unwrap();
    assert!(json.contains("\"version\":\"0.3.0\""));
    assert!(json.contains("\"traits\":"));
}

#[test]
fn legacy_conversion() {
    let legacy = LegacyEnvSense::default();
    let new = EnvSense::from_legacy(&legacy);
    let back = new.to_legacy();
    assert_eq!(legacy, back);
}

#[test]
fn nested_traits_serialization() {
    let traits = NestedTraits::default();
    let json = serde_json::to_string(&traits).unwrap();
    assert!(json.contains("\"agent\":"));
    assert!(json.contains("\"terminal\":"));
}

#[test]
fn context_list_handling() {
    let mut env = EnvSense::default();
    env.contexts.push("agent".to_string());
    env.contexts.push("ide".to_string());

    let json = serde_json::to_string(&env).unwrap();
    assert!(json.contains("\"agent\""));
    assert!(json.contains("\"ide\""));
}
```

#### Test Categories

1. **Schema Serialization**: Verify JSON output matches expected structure
2. **Conversion Functions**: Test bidirectional conversion between schemas
3. **Default Values**: Ensure all structs have sensible defaults
4. **Field Mapping**: Verify legacy fields map correctly to new locations
5. **Macro Integration**: Test that `DetectionMergerDerive` works with new
   structures

## Success Criteria

- [ ] New schema structures compile and serialize correctly
- [ ] Legacy conversion functions work bidirectionally
- [ ] All existing tests pass with new schema
- [ ] Macro system supports nested structures
- [ ] Schema version bumped to 0.3.0
- [ ] JSON output matches expected structure
- [ ] Conversion functions handle all legacy fields correctly

## Dependencies

- None (foundation phase)

## Risk Mitigation

### High-Risk Areas

1. **Schema Breaking Changes**: New schema structure may break existing
   consumers
2. **Conversion Logic**: Complex mapping between old and new schemas
3. **Macro Changes**: Extending macros may introduce bugs

### Mitigation Strategies

1. **Extensive Testing**: Comprehensive test coverage for all conversion
   scenarios
2. **Backward Compatibility**: Maintain both schemas during transition period
3. **Clear Documentation**: Document all schema changes and migration paths
4. **Gradual Rollout**: Test with internal consumers before external release

## Deliverables

1. **New Schema Structures**: Complete trait and schema definitions
2. **Conversion Functions**: Bidirectional conversion between schemas
3. **Updated Macros**: Support for nested structure merging
4. **Comprehensive Tests**: Full test coverage for new functionality
5. **Documentation**: Updated schema documentation and migration guide

## Next Phase Dependencies

Phase 1 must be completed before Phase 2 can begin, as the parser and evaluation
system depend on the new schema structures. The schema changes provide the
foundation for:

- Field registry system (Phase 2)
- Detection system updates (Phase 3)
- CLI integration changes (Phase 4)

## Implementation Notes

### Code Organization

- Keep new trait structures in `src/traits/` directory
- Maintain clear separation between new and legacy schemas
- Use feature flags if needed for gradual rollout

### Performance Considerations

- Ensure conversion functions are efficient
- Monitor serialization performance with new nested structures
- Consider caching for frequently accessed trait fields

### Migration Strategy

- Provide clear examples of schema changes
- Document all field mappings between old and new schemas
- Create migration scripts for automated conversion

## Conclusion

Phase 1 establishes the foundational changes required for the CLI streamlining
effort. Success in this phase is critical for all subsequent phases, as it
provides the schema structure that the entire system will be built upon. The
focus on backward compatibility ensures that existing users can continue using
envsense while the new features are developed and tested.
