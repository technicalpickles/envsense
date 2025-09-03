# CLI Streamlining Implementation - Phase 5: Migration & Cleanup

## Overview

Phase 5 is the final phase of the CLI streamlining implementation, focused on
completing the migration to the new schema and removing all legacy code. This
phase ensures a clean codebase while providing comprehensive migration tools for
users.

## Objective

Complete migration to new schema (0.3.0) and remove all legacy code while
providing comprehensive migration tools and documentation for users.

## Prerequisites

- Phase 1: Foundation & Schema (completed)
- Phase 2: Parser & Evaluation (completed)
- Phase 3: Detection System (completed)
- Phase 4: CLI Integration (completed)

## Phase 5 Tasks Breakdown

### Task 5.1: Create Migration Tools

**Estimated Time**: 2-3 days  
**Priority**: High  
**Files**: `src/migration.rs` (new), `tests/migration.rs` (new)

#### 5.1.1 Implement Migration Tools Module

Create a comprehensive migration tools module to help users transition from
legacy syntax.

**Implementation Steps**:

1. **Create `src/migration.rs`**:

   ```rust
   pub struct MigrationTools;

   impl MigrationTools {
       /// Convert legacy predicate syntax to new syntax
       pub fn migrate_predicate(legacy: &str) -> Result<String, String>

       /// Convert legacy JSON schema to new schema
       pub fn migrate_json(legacy_json: &str) -> Result<String, String>

       /// Validate that migrated predicate produces same results
       pub fn validate_migration(legacy: &str, new: &str) -> Result<bool, String>

       /// Get migration suggestions for a legacy predicate
       pub fn suggest_migration(legacy: &str) -> Vec<String>
   }
   ```

2. **Predicate Migration Mappings**:
   - `facet:agent_id=value` → `agent.id=value`
   - `facet:ide_id=value` → `ide.id=value`
   - `facet:ci_id=value` → `ci.id=value`
   - `trait:is_interactive` → `terminal.interactive`
   - `trait:is_tty_stdin` → `terminal.stdin.tty`
   - `trait:is_tty_stdout` → `terminal.stdout.tty`
   - `trait:is_tty_stderr` → `terminal.stderr.tty`
   - `trait:is_piped_stdin` → `terminal.stdin.piped`
   - `trait:is_piped_stdout` → `terminal.stdout.piped`
   - `trait:supports_hyperlinks` → `terminal.supports_hyperlinks`

3. **JSON Schema Migration**:
   - Convert `LegacyEnvSense` to `EnvSense` format
   - Preserve all data during conversion
   - Handle edge cases (empty strings, missing fields)

4. **Migration Validation**:
   - Verify migrated predicates produce identical results
   - Test against current environment
   - Report any discrepancies

#### 5.1.2 Create Migration Tests

**File**: `tests/migration.rs`

**Test Coverage**:

- All predicate mappings work correctly
- JSON schema conversion preserves data
- Edge cases (empty values, unknown keys)
- Migration validation catches differences
- Performance of migration operations

**Success Criteria**:

- [ ] All legacy predicates can be migrated
- [ ] JSON conversion is lossless for known fields
- [ ] Migration validation works correctly
- [ ] Comprehensive test coverage (>95%)

---

### Task 5.2: Implement CLI Migration Commands

**Estimated Time**: 1-2 days  
**Priority**: High  
**Files**: `src/main.rs`, `src/cli.rs` (if separate)

#### 5.2.1 Add Migration Subcommand

**Implementation Steps**:

1. **Update CLI Structure**:

   ```rust
   #[derive(Subcommand)]
   enum Commands {
       Info(InfoArgs),
       Check(CheckCmd),
       Migrate(MigrateCmd),  // New migration command
   }

   #[derive(Args, Clone)]
   struct MigrateCmd {
       #[arg(long, value_name = "PREDICATE")]
       predicate: Option<String>,

       #[arg(long, value_name = "FILE")]
       json: Option<String>,

       #[arg(long)]
       guide: bool,

       #[arg(long)]
       validate: bool,
   }
   ```

2. **Migration Command Implementation**:

   ```rust
   fn run_migrate(args: MigrateCmd) -> Result<(), i32> {
       if let Some(predicate) = args.predicate {
           // Migrate single predicate
       } else if let Some(file) = args.json {
           // Migrate JSON file
       } else if args.guide {
           // Show migration guide
       } else {
           // Show help
       }
   }
   ```

3. **Migration Guide Content**:
   - Overview of changes
   - Syntax comparison table
   - Common migration patterns
   - Troubleshooting tips

#### 5.2.2 Integration with Existing CLI

**Implementation Steps**:

1. **Update Help Text**: Remove references to legacy syntax
2. **Error Messages**: Suggest migration for legacy syntax
3. **Deprecation Warnings**: Ensure they point to migration tools

**Success Criteria**:

- [ ] Migration command works for all use cases
- [ ] Help text is clear and comprehensive
- [ ] Integration with existing CLI is seamless

---

### Task 5.3: Remove Legacy Code

**Estimated Time**: 3-4 days  
**Priority**: High  
**Files**: Multiple (see breakdown below)

#### 5.3.1 Remove Legacy Schema Structures

**Files to Modify**:

- `src/schema/legacy.rs` - Remove entire file
- `src/schema/mod.rs` - Remove legacy exports
- `src/schema/main.rs` - Remove conversion methods

**Implementation Steps**:

1. **Phase 1: Remove Legacy Schema File**:

   ```bash
   # Remove the legacy schema file completely
   rm src/schema/legacy.rs
   ```

2. **Phase 2: Update Schema Module**:
   - Remove `pub mod legacy;` from `src/schema/mod.rs`
   - Remove `pub use legacy::*;` exports
   - Remove `LegacyEnvSense` references

3. **Phase 3: Clean Up Main Schema**:
   - Remove `to_legacy()` and `from_legacy()` methods
   - Remove `LEGACY_SCHEMA_VERSION` constant
   - Update tests to only test new schema

**Affected Files**:

- `src/schema/legacy.rs` (delete)
- `src/schema/mod.rs` (update exports)
- `src/schema/main.rs` (remove conversion methods)

#### 5.3.2 Remove Legacy Parser Logic

**Files to Modify**:

- `src/check.rs` - Remove legacy parsing functions

**Implementation Steps**:

1. **Remove Legacy Check Variants**:

   ```rust
   // Remove these from Check enum:
   // LegacyFacet { key: String, value: String },
   // LegacyTrait { key: String },
   ```

2. **Remove Legacy Parser Functions**:
   - `parse_legacy_facet()`
   - `parse_legacy_trait()`
   - Legacy branches in `parse()` function

3. **Remove Legacy Constants**:
   - `FACETS` constant
   - `TRAITS` constant (legacy list)

4. **Update Parser Logic**:
   - Remove `facet:` and `trait:` prefix handling
   - Simplify `parse()` function to only handle new syntax

#### 5.3.3 Remove Legacy Evaluation Functions

**Files to Modify**:

- `src/check.rs` - Remove legacy evaluation functions

**Implementation Steps**:

1. **Remove Legacy Evaluation Functions**:
   - `evaluate_legacy_facet()`
   - `evaluate_legacy_trait()`

2. **Update Main Evaluation Function**:

   ```rust
   pub fn evaluate(env: &EnvSense, parsed: ParsedCheck, registry: &FieldRegistry) -> EvaluationResult {
       let mut eval_result = match parsed.check {
           Check::Context(ctx) => evaluate_context(env, &ctx),
           Check::NestedField { path, value } => {
               evaluate_nested_field(env, &path, value.as_deref(), registry)
           }
           // Remove legacy cases
       };
       // ... rest of function
   }
   ```

3. **Remove Legacy Migration Functions**:
   - `migrate_legacy_facet()`
   - `migrate_legacy_trait()`
   - `parse_with_warnings()` (move warnings to migration tools)

#### 5.3.4 Remove Legacy Tests

**Implementation Steps**:

1. **Identify Legacy Tests**:
   - All tests with "legacy" in the name
   - Tests for `facet:` and `trait:` syntax
   - Schema conversion tests

2. **Remove Test Functions**:
   - `parse_legacy_facet*` tests
   - `parse_legacy_trait*` tests
   - `evaluate_legacy_*` tests
   - `legacy_conversion*` tests

3. **Update Test Utilities**:
   - Remove legacy test data creation functions
   - Update test environment setup

**Success Criteria**:

- [ ] All legacy code removed
- [ ] No compilation errors
- [ ] All remaining tests pass
- [ ] No references to legacy syntax in codebase

---

### Task 5.4: Update Documentation

**Estimated Time**: 2-3 days  
**Priority**: High  
**Files**: `README.md`, `docs/`, `CHANGELOG.md`

#### 5.4.1 Update README.md

**Implementation Steps**:

1. **Update Examples**: Replace all legacy syntax with new dot notation
2. **Remove Legacy References**: Remove mentions of `facet:` and `trait:` syntax
3. **Add Migration Section**: Brief guide on upgrading from v0.2.0
4. **Update CLI Examples**: Ensure all examples use new syntax

**Example Updates**:

```bash
# OLD (remove these examples)
envsense check facet:agent_id=cursor
envsense check trait:is_interactive

# NEW (use these instead)
envsense check agent.id=cursor
envsense check terminal.interactive
```

#### 5.4.2 Create Migration Guide

**File**: `docs/migration-guide.md` (new)

**Content Structure**:

1. **Overview**: Why migration is needed
2. **Breaking Changes**: List of all changes
3. **Migration Steps**: Step-by-step process
4. **Syntax Mapping**: Complete mapping table
5. **CLI Tools**: How to use migration commands
6. **Common Issues**: Troubleshooting guide

**Migration Mapping Table**: | Legacy Syntax | New Syntax | Notes |
|---------------|------------|-------| | `facet:agent_id=cursor` |
`agent.id=cursor` | Direct mapping | | `trait:is_interactive` |
`terminal.interactive` | Boolean field | | `facet:ci_id=github` | `ci.id=github`
| CI context |

#### 5.4.3 Update API Documentation

**Implementation Steps**:

1. **Update Rust Doc Comments**: Remove legacy references
2. **Update Schema Documentation**: Focus on new nested structure
3. **Update Field Registry Docs**: Document new field paths
4. **Update Examples**: All code examples use new syntax

#### 5.4.4 Update CHANGELOG.md

**Implementation Steps**:

1. **Add v0.3.0 Entry**: Document breaking changes
2. **Migration Guide Reference**: Link to migration documentation
3. **Deprecation Notice**: Note removal of legacy syntax

**Changelog Entry Template**:

```markdown
## [0.3.0] - 2024-XX-XX

### Breaking Changes

- Removed legacy `facet:` and `trait:` syntax
- Schema version updated to 0.3.0
- Contexts now returned as array instead of object

### Added

- New dot notation syntax (`agent.id`, `terminal.interactive`)
- Migration tools (`envsense migrate`)
- Comprehensive migration guide

### Removed

- Legacy schema structures (`Contexts`, `Facets`, `Traits`)
- Legacy parser and evaluation functions
- Backward compatibility layers

### Migration

See [Migration Guide](docs/migration-guide.md) for upgrade instructions.
```

**Success Criteria**:

- [ ] All documentation uses new syntax
- [ ] Migration guide is comprehensive
- [ ] API documentation is updated
- [ ] CHANGELOG reflects all changes

---

### Task 5.5: Final Testing & Validation

**Estimated Time**: 2-3 days  
**Priority**: Critical  
**Files**: `tests/`, `scripts/` (validation scripts)

#### 5.5.1 Comprehensive Test Suite

**Implementation Steps**:

1. **Migration Completeness Tests**:

   ```rust
   #[test]
   fn migration_completeness() {
       // Test all known legacy predicates can be migrated
       let legacy_predicates = vec![
           "facet:agent_id=cursor",
           "facet:ide_id=vscode",
           "trait:is_interactive",
           // ... all legacy predicates
       ];

       for legacy in legacy_predicates {
           let migrated = MigrationTools::migrate_predicate(legacy).unwrap();
           // Verify migrated predicate works with current system
       }
   }
   ```

2. **End-to-End Integration Tests**:
   - Full detection pipeline works
   - CLI commands produce expected output
   - JSON schema is valid and complete

3. **Performance Regression Tests**:
   - Detection speed unchanged
   - Memory usage within bounds
   - CLI responsiveness maintained

#### 5.5.2 Real Environment Testing

**Implementation Steps**:

1. **Create Test Environments**:
   - Agent environments (Cursor, VS Code)
   - CI environments (GitHub Actions, GitLab)
   - Terminal environments (interactive, non-interactive)

2. **Validation Scripts**:

   ```bash
   #!/usr/bin/env bash
   # validate-migration.sh
   # Test migration tools against real environments

   # Test predicate migration
   envsense migrate --predicate "facet:agent_id=cursor"

   # Test JSON migration
   envsense info --json > legacy.json
   envsense migrate --json legacy.json
   ```

3. **Manual Testing Checklist**:
   - [ ] All README examples work
   - [ ] Migration commands work correctly
   - [ ] Error messages are helpful
   - [ ] Performance is acceptable

#### 5.5.3 Documentation Validation

**Implementation Steps**:

1. **Example Verification**:
   - Run all examples in README
   - Verify output matches documentation
   - Test migration guide steps

2. **Link Validation**:
   - All internal links work
   - External references are valid
   - Migration guide is complete

**Success Criteria**:

- [ ] All tests pass
- [ ] No performance regression
- [ ] Documentation examples work
- [ ] Migration tools validated in real environments

---

## Implementation Timeline

### Week 1: Migration Tools & CLI Commands

- **Days 1-3**: Task 5.1 - Migration Tools Implementation
- **Days 4-5**: Task 5.2 - CLI Migration Commands

### Week 2: Legacy Code Removal

- **Days 1-2**: Task 5.3.1 & 5.3.2 - Remove Schema & Parser Legacy
- **Days 3-4**: Task 5.3.3 & 5.3.4 - Remove Evaluation & Tests Legacy
- **Day 5**: Code cleanup and compilation fixes

### Week 3: Documentation & Testing

- **Days 1-3**: Task 5.4 - Update All Documentation
- **Days 4-5**: Task 5.5 - Final Testing & Validation

**Total Estimated Time**: 3 weeks

## Risk Management

### High-Risk Areas

1. **Breaking Changes**: Complete removal of legacy syntax
2. **Migration Complexity**: Users may have complex legacy configurations
3. **Data Loss**: JSON migration must be lossless
4. **Performance**: Ensure no regressions

### Mitigation Strategies

1. **Comprehensive Testing**: Extensive test coverage for all scenarios
2. **Migration Validation**: Tools to verify migration correctness
3. **Clear Documentation**: Step-by-step migration guide
4. **Rollback Plan**: Document how to revert if needed (downgrade to v0.2.x)

### Success Metrics

#### Technical Metrics

- [ ] Zero compilation errors
- [ ] All tests pass (target: 100%)
- [ ] No performance regression (±5%)
- [ ] Schema version 0.3.0 active

#### User Experience Metrics

- [ ] Migration tools work for all use cases
- [ ] Documentation is clear and complete
- [ ] No critical issues reported during transition
- [ ] Positive community feedback

#### Code Quality Metrics

- [ ] No legacy code references
- [ ] Reduced codebase complexity
- [ ] Improved maintainability
- [ ] Clean architecture

## Deliverables

### Code Deliverables

1. **Migration Tools** (`src/migration.rs`)
2. **CLI Migration Commands** (updated `src/main.rs`)
3. **Clean Codebase** (all legacy code removed)
4. **Comprehensive Tests** (migration and integration tests)

### Documentation Deliverables

1. **Updated README.md** (new syntax examples)
2. **Migration Guide** (`docs/migration-guide.md`)
3. **Updated API Documentation** (Rust docs)
4. **Updated CHANGELOG.md** (v0.3.0 entry)

### Validation Deliverables

1. **Test Suite** (comprehensive coverage)
2. **Validation Scripts** (real environment testing)
3. **Performance Benchmarks** (regression testing)
4. **Migration Validation** (correctness verification)

## Post-Phase 5 Activities

### Immediate (Week 4)

1. **Release v0.3.0**: Tag and publish new version
2. **Community Communication**: Announce breaking changes
3. **Monitor Issues**: Watch for migration problems

### Short-term (1-2 months)

1. **User Feedback**: Collect and address migration issues
2. **Documentation Updates**: Based on user questions
3. **Tool Improvements**: Enhance migration tools if needed

### Long-term (3+ months)

1. **Legacy Support End**: Completely end v0.2.x support
2. **New Features**: Build on clean v0.3.0 foundation
3. **Community Growth**: Leverage improved UX

## Conclusion

Phase 5 represents the culmination of the CLI streamlining effort. By removing
all legacy code and providing comprehensive migration tools, we ensure a clean
foundation for future development while minimizing disruption to existing users.

The success of this phase depends on:

1. **Thorough Testing**: Comprehensive validation of all changes
2. **Clear Communication**: Excellent documentation and migration guides
3. **User Support**: Responsive handling of migration issues
4. **Quality Assurance**: No regressions in functionality or performance

Upon completion, envsense will have a significantly improved CLI interface,
cleaner codebase, and better foundation for future enhancements.
