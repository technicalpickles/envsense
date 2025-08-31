# Declarative Detector Consolidation Planning

This directory contains comprehensive planning documents for consolidating and
enhancing the declarative detector system in envsense.

## Documents Overview

### 📋 [Declarative Detector Consolidation Plan](./declarative-detector-consolidation.md)

**Primary planning document** outlining the complete consolidation strategy.

**Key Topics:**

- Current state analysis and implementation comparison
- Common patterns and differences between detectors
- Four-phase consolidation approach
- Code reduction opportunities (50-60% target)
- Risk assessment and mitigation strategies

**Phases:**

1. **Extract Common Utilities** (Low Risk, High Value)
2. **Standardize Override System** (Medium Risk, High Value)
3. **Standardize Selection Logic** (Medium Risk, Medium Value)
4. **Create Base Trait** (High Risk, High Value)

### 🔄 [Contextual Value Extraction](./contextual-value-extraction.md)

**Advanced enhancement** for declarative value extraction from environment
variables.

**Key Topics:**

- CI-specific value mappings (branch, PR status, build numbers)
- Value transformations (boolean, integer, string operations)
- Declarative fallback chains (no more hardcoded Rust logic)
- Extensible transformation system

**Phases:**

1. **Core Infrastructure** (Low Risk, High Value)
2. **CI Value Mappings** (Medium Risk, High Value)
3. **Integration and Testing** (Medium Risk, Medium Value)
4. **Advanced Features** (High Risk, High Value)

### 🔧 [Override System Design](./override-system-design.md)

**Detailed design** for implementing a comprehensive override system across all
detectors.

**Key Features:**

- Consistent override pattern: `ENVSENSE_{DETECTOR}=<value>`
- Semantic overrides: `ENVSENSE_ASSUME_{MODE}=1`
- Backward compatibility with existing agent overrides
- Support for testing, debugging, and custom environments

**New Override Variables:**

```bash
# IDE Detection
ENVSENSE_IDE=none                    # Disable IDE detection
ENVSENSE_IDE=custom-editor           # Force specific IDE
ENVSENSE_ASSUME_TERMINAL=1           # Semantic disable

# CI Detection
ENVSENSE_CI=none                     # Disable CI detection
ENVSENSE_CI=custom-ci                # Force specific CI
ENVSENSE_ASSUME_LOCAL=1              # Semantic disable

# Agent Detection (existing, maintained)
ENVSENSE_AGENT=none                  # Disable agent detection
ENVSENSE_AGENT=custom-agent          # Force specific agent
ENVSENSE_ASSUME_HUMAN=1              # Semantic disable
```

### 🧪 [Testing Strategy](./testing-strategy-consolidation.md)

**Comprehensive testing approach** ensuring quality and preventing regressions.

**Testing Phases:**

- **Phase 1**: Utility function testing (95% coverage target)
- **Phase 2**: Override system testing (90% coverage target)
- **Phase 3**: Selection logic migration testing (95% coverage target)
- **Phase 4**: Base trait implementation testing (90% coverage target)

**Quality Gates:**

- All existing tests must pass (no regression)
- New functionality must have 90%+ coverage
- Performance must not regress by >10%
- Memory usage must not increase by >20%
- CLI behavior must remain identical

### 🔄 [Contextual Value Extraction](./contextual-value-extraction.md)

**Advanced enhancement** for declarative value extraction from environment
variables.

**Key Features:**

- CI-specific value mappings (branch, PR status, build numbers)
- Value transformations (boolean, integer, string operations)
- Declarative fallback chains (no more hardcoded Rust logic)
- Extensible transformation system

**Implementation Phases:**

- **Phase 1**: Core infrastructure and data structures
- **Phase 2**: CI value mappings for all major systems
- **Phase 3**: Integration with existing detection pipeline
- **Phase 4**: Advanced features and optimizations

**Benefits:**

- Eliminates hardcoded fallback chains in Rust code
- Makes CI detection fully declarative
- Enables easy addition of new CI systems
- Improves maintainability and testability

## Project Summary

### Current State

- **3 declarative detectors** with similar but inconsistent implementations
- **Agent detector** has overrides, IDE/CI detectors don't
- **Mixed selection logic** (priority vs confidence-based)
- **Code duplication** in evidence generation and mapping iteration

### Proposed Improvements

#### 🎯 Code Consolidation

- **50-60% code reduction** through shared utilities and base traits
- **Consistent behavior** across all detectors
- **Easier maintenance** with centralized logic

#### 🔧 Enhanced Override System

- **Uniform override pattern** across all detector types
- **Better testing capabilities** with disable/force options
- **Custom environment support** for proprietary systems

#### 📊 Standardized Selection Logic

- **Priority-based selection** for all detectors
- **Predictable behavior** with clear ordering rules
- **Conflict resolution** for overlapping detectors

#### 🏗️ Improved Architecture

- **Base trait system** for common functionality
- **Easy extensibility** for new detector types
- **Configuration-driven** detection (future enhancement)

### Implementation Timeline

| **Phase**                    | **Duration** | **Risk** | **Value** |
| ---------------------------- | ------------ | -------- | --------- |
| **Phase 1: Utilities**       | 1 week       | Low      | High      |
| **Phase 2: Overrides**       | 1 week       | Low      | High      |
| **Phase 3: Selection Logic** | 1 week       | Medium   | Medium    |
| **Phase 4: Base Trait**      | 1 week       | High     | High      |

### Success Metrics

- ✅ **50-60% code reduction** in detector implementations
- ✅ **100% test coverage** maintained throughout migration
- ✅ **No performance regression** (within 10% of current performance)
- ✅ **Backward compatibility** for all existing functionality
- ✅ **Enhanced testing capabilities** through override system

### Benefits

#### For Developers

- **Reduced code duplication** makes maintenance easier
- **Consistent patterns** across all detectors
- **Better testing tools** for debugging and validation
- **Easier to add new detectors** with base trait system

#### For Users

- **More reliable detection** through standardized logic
- **Better debugging capabilities** with override system
- **Support for custom environments** through overrides
- **Consistent CLI behavior** across all detection types

#### For Testing

- **Comprehensive override system** enables edge case testing
- **Disable specific detectors** for isolation testing
- **Force custom values** for integration testing
- **Semantic overrides** for clear test intent

## Next Steps

1. **Review planning documents** and gather feedback
2. **Create feature branch** for consolidation work
3. **Implement Phase 1** (utilities) with comprehensive tests
4. **Validate approach** before proceeding to subsequent phases
5. **Iterate based on learnings** from each phase

## Related Documentation

- [Architecture Overview](../architecture.md) - System design and core concepts
- [Testing Guidelines](../testing.md) - General testing strategy
- [Extending Guide](../extending.md) - Adding new detectors

---

**Note**: This consolidation project represents a significant architectural
improvement that will make the envsense detection system more maintainable,
testable, and extensible while preserving all existing functionality.
