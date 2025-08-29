# Contextual Value Extraction Planning

This document outlines the plan for implementing contextual value extraction in the `env_mapping.rs` system, enabling CI-specific value mappings that are applied only when a particular CI system is detected.

## Problem Statement

The current `env_mapping.rs` system excels at **detection** (determining which CI system is present) but lacks **value extraction** capabilities. This forces complex fallback logic to be implemented in Rust code rather than declaratively.

### Current Limitations

1. **No contextual value extraction**: Can't specify "when GitHub Actions is detected, extract branch from `GITHUB_REF_NAME`"
2. **Hardcoded fallback chains**: Branch detection requires Rust code like:
   ```rust
   snap.get_env("GITHUB_REF_NAME")
       .or_else(|| snap.get_env("CI_COMMIT_REF_NAME"))
       .or_else(|| snap.get_env("CIRCLE_BRANCH"))
       // ... more fallbacks
   ```
3. **CI-specific logic scattered**: Each CI system's value extraction logic is embedded in detector code
4. **Difficult to extend**: Adding new CI systems requires both detection and extraction code changes

### Target State

A declarative system where each CI mapping defines:
- **Detection criteria** (which env vars indicate this CI system)
- **Value mappings** (which env vars contain which values when this CI is detected)
- **Transformations** (how to process extracted values)

## Design Overview

### Core Concept

Extend `EnvMapping` to include **contextual value mappings** that are only applied when that specific environment is detected.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvMapping {
    // ... existing fields ...
    
    /// Value mappings specific to this environment (only applied when this mapping matches)
    #[serde(default)]
    pub value_mappings: Vec<ValueMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueMapping {
    /// The key this value will be stored under in the result
    pub target_key: String,
    /// The environment variable to extract the value from
    pub source_key: String,
    /// Whether this value extraction is required
    #[serde(default)]
    pub required: bool,
    /// Transformation to apply to the value
    #[serde(default)]
    pub transform: Option<ValueTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueTransform {
    /// Convert to boolean (non-empty = true, empty = false)
    ToBool,
    /// Convert to lowercase
    ToLowercase,
    /// Check if equals specific value, return boolean
    Equals(String),
    /// Check if contains substring, return boolean
    Contains(String),
    /// Parse as integer
    ToInt,
    /// Custom transformation function
    Custom(String),
}
```

### Example Implementation

```rust
// GitHub Actions with value mappings
EnvMapping {
    id: "github-actions".to_string(),
    confidence: HIGH,
    indicators: vec![EnvIndicator {
        key: "GITHUB_ACTIONS".to_string(),
        value: None,
        required: false,
        // ...
    }],
    facets: HashMap::from([("ci_id".to_string(), "github_actions".to_string())]),
    contexts: vec!["ci".to_string()],
    value_mappings: vec![
        ValueMapping {
            target_key: "branch".to_string(),
            source_key: "GITHUB_REF_NAME".to_string(),
            required: false,
            transform: None,
        },
        ValueMapping {
            target_key: "is_pr".to_string(),
            source_key: "GITHUB_EVENT_NAME".to_string(),
            required: false,
            transform: Some(ValueTransform::Equals("pull_request".to_string())),
        },
        ValueMapping {
            target_key: "pr_number".to_string(),
            source_key: "GITHUB_EVENT_NUMBER".to_string(),
            required: false,
            transform: Some(ValueTransform::ToInt),
        },
    ],
}

// GitLab CI with value mappings
EnvMapping {
    id: "gitlab-ci".to_string(),
    confidence: HIGH,
    indicators: vec![EnvIndicator {
        key: "GITLAB_CI".to_string(),
        value: None,
        required: false,
        // ...
    }],
    facets: HashMap::from([("ci_id".to_string(), "gitlab_ci".to_string())]),
    contexts: vec!["ci".to_string()],
    value_mappings: vec![
        ValueMapping {
            target_key: "branch".to_string(),
            source_key: "CI_COMMIT_REF_NAME".to_string(),
            required: false,
            transform: None,
        },
        ValueMapping {
            target_key: "is_pr".to_string(),
            source_key: "CI_MERGE_REQUEST_ID".to_string(),
            required: false,
            transform: Some(ValueTransform::ToBool),
        },
    ],
}
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1)

#### 1.1 Extend Data Structures
- [ ] Add `ValueMapping` struct to `env_mapping.rs`
- [ ] Add `ValueTransform` enum with common transformations
- [ ] Add `value_mappings` field to `EnvMapping`
- [ ] Update serialization/deserialization

#### 1.2 Implement Value Extraction Logic
- [ ] Add `extract_values()` method to `EnvMapping`
- [ ] Implement transformation functions
- [ ] Add error handling for malformed values
- [ ] Add logging for debugging

#### 1.3 Update Detection Engine
- [ ] Modify `DeclarativeDetector` trait to support value extraction
- [ ] Update detection pipeline to apply value mappings
- [ ] Ensure backward compatibility

### Phase 2: CI Value Mappings (Week 2)

#### 2.1 GitHub Actions Mappings
- [ ] Branch: `GITHUB_REF_NAME`
- [ ] PR status: `GITHUB_EVENT_NAME` → `equals("pull_request")`
- [ ] PR number: `GITHUB_EVENT_NUMBER` → `to_int`
- [ ] Repository: `GITHUB_REPOSITORY`
- [ ] Workflow: `GITHUB_WORKFLOW`

#### 2.2 GitLab CI Mappings
- [ ] Branch: `CI_COMMIT_REF_NAME`
- [ ] PR status: `CI_MERGE_REQUEST_ID` → `to_bool`
- [ ] Pipeline ID: `CI_PIPELINE_ID` → `to_int`
- [ ] Project path: `CI_PROJECT_PATH`

#### 2.3 CircleCI Mappings
- [ ] Branch: `CIRCLE_BRANCH`
- [ ] PR status: `CIRCLE_PR_NUMBER` → `to_bool`
- [ ] Build number: `CIRCLE_BUILD_NUM` → `to_int`
- [ ] Project name: `CIRCLE_PROJECT_REPONAME`

#### 2.4 Generic CI Mappings
- [ ] Branch: `BRANCH_NAME`, `GIT_BRANCH` (fallback)
- [ ] PR status: `CI_PULL_REQUEST` → `contains("true")`
- [ ] Build number: `BUILD_NUMBER` → `to_int`

### Phase 3: Integration and Testing (Week 3)

#### 3.1 Update CI Declarative Detector
- [ ] Remove hardcoded value extraction logic
- [ ] Use new value mapping system
- [ ] Maintain existing API compatibility
- [ ] Update tests to use new system

#### 3.2 Comprehensive Testing
- [ ] Unit tests for each transformation type
- [ ] Integration tests for each CI system
- [ ] Edge case testing (missing values, malformed data)
- [ ] Performance testing

#### 3.3 Documentation and Examples
- [ ] Update `env_mapping.rs` documentation
- [ ] Add examples for common CI systems
- [ ] Create migration guide for existing mappings
- [ ] Update architecture documentation

### Phase 4: Advanced Features (Week 4)

#### 4.1 Conditional Value Mappings
- [ ] Support for conditional extraction based on other values
- [ ] Example: Extract PR number only when `is_pr` is true

#### 4.2 Custom Transformations
- [ ] Plugin system for custom transformation functions
- [ ] Support for complex data parsing
- [ ] JSON parsing for structured environment variables

#### 4.3 Validation and Error Handling
- [ ] Schema validation for value mappings
- [ ] Graceful handling of missing or invalid values
- [ ] Detailed error messages for debugging

## Technical Details

### Value Transformation Functions

```rust
impl ValueTransform {
    pub fn apply(&self, value: &str) -> Result<serde_json::Value, String> {
        match self {
            ValueTransform::ToBool => {
                Ok(json!(!value.is_empty()))
            }
            ValueTransform::ToLowercase => {
                Ok(json!(value.to_lowercase()))
            }
            ValueTransform::Equals(target) => {
                Ok(json!(value == target))
            }
            ValueTransform::Contains(substring) => {
                Ok(json!(value.to_lowercase().contains(&substring.to_lowercase())))
            }
            ValueTransform::ToInt => {
                value.parse::<i64>()
                    .map(json!)
                    .map_err(|e| format!("Failed to parse '{}' as integer: {}", value, e))
            }
            ValueTransform::Custom(func_name) => {
                // Future: plugin system for custom transformations
                Err(format!("Custom transformation '{}' not implemented", func_name))
            }
        }
    }
}
```

### Integration with Detection Pipeline

```rust
impl EnvMapping {
    pub fn extract_values(&self, env_vars: &HashMap<String, String>) -> HashMap<String, serde_json::Value> {
        let mut extracted = HashMap::new();
        
        for mapping in &self.value_mappings {
            if let Some(value) = env_vars.get(&mapping.source_key) {
                match mapping.transform.as_ref() {
                    Some(transform) => {
                        match transform.apply(value) {
                            Ok(transformed) => {
                                extracted.insert(mapping.target_key.clone(), transformed);
                            }
                            Err(e) => {
                                // Log error but continue with other mappings
                                log::warn!("Failed to transform {}: {}", mapping.source_key, e);
                            }
                        }
                    }
                    None => {
                        extracted.insert(mapping.target_key.clone(), json!(value));
                    }
                }
            } else if mapping.required {
                log::warn!("Required value mapping missing: {}", mapping.source_key);
            }
        }
        
        extracted
    }
}
```

## Migration Strategy

### Backward Compatibility

1. **Existing mappings continue to work** - no changes required
2. **Value mappings are optional** - can be added incrementally
3. **Detection logic unchanged** - only value extraction is enhanced

### Gradual Migration

1. **Phase 1**: Add value mappings to existing CI mappings
2. **Phase 2**: Update CI declarative detector to use new system
3. **Phase 3**: Remove old hardcoded extraction logic
4. **Phase 4**: Add value mappings to other detector types

### Testing Strategy

1. **Unit tests** for each transformation type
2. **Integration tests** for complete CI detection + extraction
3. **Snapshot tests** to ensure output consistency
4. **Performance tests** to ensure no regression

## Benefits

### For Developers

1. **Declarative configuration** - no more hardcoded fallback chains
2. **Easier maintenance** - all CI-specific logic in one place
3. **Better testability** - value extraction can be tested independently
4. **Extensibility** - easy to add new CI systems or value mappings

### For Users

1. **More reliable detection** - consistent value extraction across CI systems
2. **Better debugging** - clear mapping between env vars and extracted values
3. **Custom environments** - easy to add support for proprietary CI systems

### For Testing

1. **Comprehensive coverage** - test each CI system's value extraction
2. **Edge case handling** - test missing values, malformed data
3. **Performance validation** - ensure no performance regression

## Success Metrics

- ✅ **100% of CI value extraction** moved to declarative mappings
- ✅ **Zero hardcoded fallback chains** in Rust code
- ✅ **All existing tests pass** with no regressions
- ✅ **Performance within 5%** of current implementation
- ✅ **Support for all major CI systems** with comprehensive value mappings

## Future Enhancements

### Advanced Value Mappings

1. **Conditional extraction** based on other environment variables
2. **Complex transformations** (JSON parsing, regex matching)
3. **Default values** for missing environment variables
4. **Validation rules** for extracted values

### Extended Support

1. **Agent value mappings** (extract agent-specific information)
2. **IDE value mappings** (extract IDE-specific settings)
3. **Terminal value mappings** (extract terminal capabilities)
4. **Custom environment mappings** (user-defined value extractions)

### Performance Optimizations

1. **Lazy evaluation** of value mappings
2. **Caching** of transformation results
3. **Parallel processing** of multiple mappings
4. **Compiled transformations** for common patterns

## Conclusion

Contextual value extraction represents a significant enhancement to the `env_mapping.rs` system, enabling fully declarative environment detection and value extraction. This approach eliminates hardcoded logic, improves maintainability, and provides a foundation for future enhancements.

The phased implementation approach ensures backward compatibility while gradually migrating to the new system, minimizing risk while maximizing value.
