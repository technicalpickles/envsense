# CLI Streamlining Implementation - Phase 3: Detection System

## Overview

Phase 3 focuses on updating the detection system to populate the new nested
trait structure introduced in Phase 1. This phase transforms how detectors
output their results and ensures the macro system can properly merge nested
objects.

## Prerequisites

- **Phase 1**: New schema structures (`NestedTraits`, `AgentTraits`, etc.) must
  be implemented
- **Phase 2**: Field registry and parser must be functional for testing

## Objectives

1. Update all detectors to output nested trait structures
2. Enhance macro system to handle nested object merging
3. Update engine to work with new schema
4. Maintain backward compatibility during transition
5. Ensure evidence collection works with new field paths

## Task Breakdown

### Task 3.1: Update Terminal Detector for Nested Structure

**Priority**: High  
**Estimated Time**: 1-2 days  
**Files**: `src/detectors/terminal.rs`

#### Current State Analysis

The terminal detector currently outputs flat trait keys:

- `is_interactive`, `is_tty_stdin`, `is_tty_stdout`, etc.
- Uses individual `traits_patch.insert()` calls for each field

#### Required Changes

1. **Replace flat trait keys with nested structure**:

   ```rust
   // Instead of individual flat keys, create nested TerminalTraits object
   let terminal_traits = TerminalTraits {
       interactive: snap.is_tty_stdin() && snap.is_tty_stdout(),
       color_level: detect_color_level(snap),
       stdin: StreamInfo {
           tty: snap.is_tty_stdin(),
           piped: !snap.is_tty_stdin(),
       },
       stdout: StreamInfo {
           tty: snap.is_tty_stdout(),
           piped: !snap.is_tty_stdout(),
       },
       stderr: StreamInfo {
           tty: snap.is_tty_stderr(),
           piped: !snap.is_tty_stderr(),
       },
       supports_hyperlinks: detect_hyperlinks(snap),
   };
   ```

2. **Update traits_patch insertion**:

   ```rust
   // Insert as nested object under "terminal" key
   detection.traits_patch.insert(
       "terminal".to_string(),
       serde_json::to_value(terminal_traits).unwrap(),
   );
   ```

3. **Update evidence field paths**:

   ```rust
   // Update evidence to reference nested field paths
   detection.evidence.push(
       Evidence::tty_trait("terminal.stdin.tty", snap.is_tty_stdin())
           .with_supports(vec!["terminal.stdin.tty".into()])
           .with_confidence(TERMINAL),
   );
   ```

4. **Maintain backward compatibility**:
   ```rust
   // Keep legacy flat keys for transition period
   detection.traits_patch.insert("is_interactive".to_string(), json!(terminal_traits.interactive));
   detection.traits_patch.insert("is_tty_stdin".to_string(), json!(terminal_traits.stdin.tty));
   // ... other legacy keys
   ```

#### Testing Requirements

- [ ] Verify nested TerminalTraits object is created correctly
- [ ] Confirm JSON serialization produces expected nested structure
- [ ] Test evidence collection with new field paths
- [ ] Ensure backward compatibility with legacy flat keys
- [ ] Validate color level enum serialization
- [ ] Test hyperlinks detection integration

#### Success Criteria

- [ ] Terminal detector outputs nested `terminal` object in traits_patch
- [ ] All terminal-related tests pass
- [ ] Evidence references use nested field paths (`terminal.interactive`, etc.)
- [ ] Legacy flat keys still present for compatibility
- [ ] JSON output matches new schema structure

---

### Task 3.2: Update Agent Detector for Nested Structure

**Priority**: High  
**Estimated Time**: 1 day  
**Files**: `src/detectors/agent_declarative.rs`

#### Current State Analysis

The agent detector currently:

- Uses `agent.id` key in traits_patch (partially nested)
- Maintains `agent_id` in facets_patch for backward compatibility
- Adds `agent` to contexts_add

#### Required Changes

1. **Create nested AgentTraits object**:

   ```rust
   if let Some(agent_id) = detect_agent_id(snap) {
       let agent_traits = AgentTraits {
           id: Some(agent_id.clone()),
       };

       detection.traits_patch.insert(
           "agent".to_string(),
           serde_json::to_value(agent_traits).unwrap(),
       );
   }
   ```

2. **Update evidence field paths**:

   ```rust
   detection.evidence.push(
       Evidence::env_var("CURSOR_AGENT", "1")
           .with_supports(vec!["agent.id".into()])
           .with_confidence(mapping.confidence),
   );
   ```

3. **Maintain legacy facets**:
   ```rust
   // Keep for backward compatibility
   detection.facets_patch.insert("agent_id".to_string(), json!(agent_id));
   ```

#### Testing Requirements

- [ ] Test all agent detection scenarios (cursor, replit, aider, etc.)
- [ ] Verify nested agent object structure
- [ ] Confirm evidence uses correct field paths
- [ ] Test override scenarios (ENVSENSE_AGENT, ENVSENSE_ASSUME_HUMAN)
- [ ] Validate host detection still works

#### Success Criteria

- [ ] Agent detector outputs nested `agent` object
- [ ] All agent detection tests pass
- [ ] Evidence references use `agent.id` field path
- [ ] Legacy `agent_id` facet maintained for compatibility
- [ ] Context detection unchanged

---

### Task 3.3: Update IDE Detector for Nested Structure

**Priority**: High  
**Estimated Time**: 1 day  
**Files**: `src/detectors/ide_declarative.rs`

#### Current State Analysis

Similar to agent detector, needs to be updated to use nested structure.

#### Required Changes

1. **Create nested IdeTraits object**:

   ```rust
   if let Some(ide_id) = detect_ide_id(snap) {
       let ide_traits = IdeTraits {
           id: Some(ide_id.clone()),
       };

       detection.traits_patch.insert(
           "ide".to_string(),
           serde_json::to_value(ide_traits).unwrap(),
       );
   }
   ```

2. **Update evidence and maintain compatibility**:

   ```rust
   // Evidence with nested path
   detection.evidence.push(
       Evidence::env_var("VSCODE_INJECTION", "1")
           .with_supports(vec!["ide.id".into()])
   );

   // Legacy facet
   detection.facets_patch.insert("ide_id".to_string(), json!(ide_id));
   ```

#### Testing Requirements

- [ ] Test IDE detection scenarios
- [ ] Verify nested structure
- [ ] Confirm evidence field paths
- [ ] Test override scenarios

#### Success Criteria

- [ ] IDE detector outputs nested `ide` object
- [ ] All IDE tests pass
- [ ] Evidence uses `ide.id` field path
- [ ] Legacy compatibility maintained

---

### Task 3.4: Update CI Detector for Nested Structure

**Priority**: High  
**Estimated Time**: 1-2 days  
**Files**: `src/detectors/ci_declarative.rs`

#### Current State Analysis

CI detector is more complex as it needs to populate multiple CI trait fields.

#### Required Changes

1. **Create comprehensive CiTraits object**:

   ```rust
   if let Some(ci_info) = detect_ci_info(snap) {
       let ci_traits = CiTraits {
           id: ci_info.id,
           vendor: ci_info.vendor,
           name: ci_info.name,
           is_pr: ci_info.is_pr,
           branch: ci_info.branch,
       };

       detection.traits_patch.insert(
           "ci".to_string(),
           serde_json::to_value(ci_traits).unwrap(),
       );
   }
   ```

2. **Update evidence for all CI fields**:

   ```rust
   // Evidence for different CI aspects
   detection.evidence.push(
       Evidence::env_var("GITHUB_ACTIONS", "true")
           .with_supports(vec!["ci.id".into(), "ci.vendor".into()])
   );

   if let Some(pr_number) = snap.env_vars.get("GITHUB_PR_NUMBER") {
       detection.evidence.push(
           Evidence::env_var("GITHUB_PR_NUMBER", pr_number)
               .with_supports(vec!["ci.is_pr".into()])
       );
   }
   ```

3. **Maintain legacy facets**:
   ```rust
   // Keep legacy CI facet
   detection.facets_patch.insert("ci_id".to_string(), json!(ci_info.id));
   ```

#### Testing Requirements

- [ ] Test all CI providers (GitHub, GitLab, CircleCI, etc.)
- [ ] Verify all CI trait fields are populated
- [ ] Test PR detection scenarios
- [ ] Test branch detection
- [ ] Verify evidence for different CI aspects

#### Success Criteria

- [ ] CI detector outputs complete nested `ci` object
- [ ] All CI trait fields populated correctly
- [ ] Evidence references appropriate nested paths
- [ ] All CI detection tests pass
- [ ] Legacy compatibility maintained

---

### Task 3.5: Enhance Macro System for Nested Object Merging

**Priority**: Critical  
**Estimated Time**: 2-3 days  
**Files**: `envsense-macros/envsense-macros-impl/src/lib.rs`

#### Current State Analysis

The macro system currently:

- Handles flat trait merging well
- Has some nested trait support but needs enhancement
- Needs to handle nested object merging from traits_patch

#### Required Changes

1. **Enhance nested trait merging logic**:

   ```rust
   // Update generate_nested_trait_merge to handle object-based patches
   fn generate_nested_trait_merge(field_name: &syn::Ident) -> proc_macro2::TokenStream {
       quote! {
           // Handle nested object merging from traits_patch
           if let Some(agent_obj) = all_traits.get("agent").and_then(|v| v.as_object()) {
               if let Some(id) = agent_obj.get("id").and_then(|v| v.as_str()) {
                   self.#field_name.agent.id = Some(id.to_string());
               }
           }

           if let Some(terminal_obj) = all_traits.get("terminal").and_then(|v| v.as_object()) {
               if let Some(interactive) = terminal_obj.get("interactive").and_then(|v| v.as_bool()) {
                   self.#field_name.terminal.interactive = interactive;
               }
               // Handle nested StreamInfo objects
               if let Some(stdin_obj) = terminal_obj.get("stdin").and_then(|v| v.as_object()) {
                   if let Some(tty) = stdin_obj.get("tty").and_then(|v| v.as_bool()) {
                       self.#field_name.terminal.stdin.tty = tty;
                   }
                   if let Some(piped) = stdin_obj.get("piped").and_then(|v| v.as_bool()) {
                       self.#field_name.terminal.stdin.piped = piped;
                   }
               }
               // Similar for stdout, stderr
           }

           // Maintain backward compatibility with flat keys
           if !all_traits.contains_key("agent") {
               if let Some(value) = all_traits.get("agent.id").and_then(|v| v.as_str()) {
                   self.#field_name.agent.id = Some(value.to_string());
               }
           }
       }
   }
   ```

2. **Add support for partial object merging**:

   ```rust
   // Handle cases where only some fields of a nested object are provided
   fn merge_partial_nested_object() -> proc_macro2::TokenStream {
       quote! {
           // Merge partial updates to nested objects
           // This handles cases where detectors provide individual field updates
           // rather than complete nested objects
       }
   }
   ```

3. **Enhance error handling**:
   ```rust
   // Add better error handling for malformed nested objects
   // Log warnings for unexpected structure but continue processing
   ```

#### Testing Requirements

- [ ] Test nested object merging with complete objects
- [ ] Test partial object merging
- [ ] Test backward compatibility with flat keys
- [ ] Test error handling with malformed data
- [ ] Verify performance with complex nested structures

#### Success Criteria

- [ ] Macro correctly merges nested trait objects
- [ ] Handles both object-based and flat key-based patches
- [ ] Maintains backward compatibility
- [ ] All macro-generated tests pass
- [ ] No performance regression

---

### Task 3.6: Update Detection Engine for New Schema

**Priority**: High  
**Estimated Time**: 1 day  
**Files**: `src/engine.rs`

#### Current State Analysis

The engine currently:

- Uses new schema structure (EnvSense with NestedTraits)
- Collects detections and merges them
- Needs verification that it works with updated detectors

#### Required Changes

1. **Verify detection collection works with nested objects**:

   ```rust
   // Ensure the engine properly handles nested trait patches
   let detections: Vec<envsense_macros::Detection> = self
       .detectors
       .iter()
       .map(|detector| {
           let detection = detector.detect(snapshot);
           envsense_macros::Detection {
               contexts_add: detection.contexts_add,
               traits_patch: detection.traits_patch, // Now contains nested objects
               facets_patch: detection.facets_patch, // Legacy support
               evidence: detection.evidence.into_iter().map(|e| serde_json::to_value(e).unwrap()).collect(),
               confidence: detection.confidence,
           }
       })
       .collect();
   ```

2. **Add validation for nested structure**:
   ```rust
   // Optional: Add validation that merged result has expected structure
   fn validate_nested_structure(result: &EnvSense) -> Result<(), String> {
       // Validate that nested traits are properly structured
       // This is mainly for debugging during development
   }
   ```

#### Testing Requirements

- [ ] Test engine with all updated detectors
- [ ] Verify nested trait merging works end-to-end
- [ ] Test with multiple detectors providing overlapping data
- [ ] Verify evidence collection and merging

#### Success Criteria

- [ ] Engine successfully merges nested trait objects
- [ ] All integration tests pass
- [ ] JSON output has correct nested structure
- [ ] Evidence is properly collected and merged

---

### Task 3.7: Update Evidence Collection for Nested Paths

**Priority**: Medium  
**Estimated Time**: 1 day  
**Files**: `src/schema/evidence.rs`, detector files

#### Current State Analysis

Evidence currently uses flat field names in `supports` field.

#### Required Changes

1. **Update evidence creation to use nested paths**:

   ```rust
   // Update all evidence creation to use nested field paths
   Evidence::env_var("CURSOR_AGENT", "1")
       .with_supports(vec!["agent.id".into()]) // Instead of "agent_id"

   Evidence::tty_trait("terminal.stdin.tty", true)
       .with_supports(vec!["terminal.stdin.tty".into()]) // Instead of "is_tty_stdin"
   ```

2. **Add helper methods for common evidence patterns**:
   ```rust
   impl Evidence {
       pub fn agent_detection(env_var: &str, value: &str, agent_id: &str) -> Self {
           Self::env_var(env_var, value)
               .with_supports(vec!["agent.id".into()])
               .with_description(format!("Detected {} agent", agent_id))
       }

       pub fn terminal_stream(stream: &str, is_tty: bool) -> Self {
           Self::tty_trait(&format!("terminal.{}.tty", stream), is_tty)
               .with_supports(vec![format!("terminal.{}.tty", stream).into()])
       }
   }
   ```

#### Testing Requirements

- [ ] Verify evidence uses correct nested field paths
- [ ] Test evidence helper methods
- [ ] Ensure evidence serialization works correctly

#### Success Criteria

- [ ] All evidence uses nested field paths in `supports`
- [ ] Evidence helper methods work correctly
- [ ] Evidence tests pass with new paths

---

### Task 3.8: Integration Testing and Validation

**Priority**: High  
**Estimated Time**: 1-2 days  
**Files**: `tests/`, integration test files

#### Required Testing

1. **End-to-end detection testing**:

   ```rust
   #[test]
   fn test_nested_trait_detection_integration() {
       let engine = DetectionEngine::new()
           .register(TerminalDetector::new())
           .register(DeclarativeAgentDetector::new())
           .register(DeclarativeCiDetector::new())
           .register(DeclarativeIdeDetector::new());

       let result = engine.detect();

       // Verify nested structure
       assert!(result.traits.terminal.interactive || !result.traits.terminal.interactive); // Field exists
       assert!(result.traits.agent.id.is_some() || result.traits.agent.id.is_none()); // Field exists

       // Verify JSON structure
       let json = serde_json::to_string_pretty(&result).unwrap();
       assert!(json.contains("\"traits\":{"));
       assert!(json.contains("\"agent\":{"));
       assert!(json.contains("\"terminal\":{"));
   }
   ```

2. **Backward compatibility testing**:

   ```rust
   #[test]
   fn test_legacy_conversion_with_nested_traits() {
       let new_env = EnvSense::detect();
       let legacy = new_env.to_legacy();
       let back_to_new = EnvSense::from_legacy(&legacy);

       // Verify roundtrip conversion preserves data
       assert_eq!(new_env.traits.agent.id, back_to_new.traits.agent.id);
       assert_eq!(new_env.traits.terminal.interactive, back_to_new.traits.terminal.interactive);
   }
   ```

3. **Performance testing**:
   ```rust
   #[test]
   fn test_nested_trait_performance() {
       let start = std::time::Instant::now();
       for _ in 0..1000 {
           let _ = EnvSense::detect();
       }
       let duration = start.elapsed();

       // Ensure no significant performance regression
       assert!(duration.as_millis() < 1000, "Detection took too long: {:?}", duration);
   }
   ```

#### Success Criteria

- [ ] All integration tests pass
- [ ] Backward compatibility maintained
- [ ] No performance regression
- [ ] JSON output structure validated
- [ ] Evidence collection works correctly

---

## Phase 3 Success Criteria

### Technical Requirements

- [ ] All detectors output nested trait structures
- [ ] Macro system correctly merges nested objects
- [ ] Engine works with new detection format
- [ ] Evidence uses nested field paths
- [ ] JSON output matches new schema structure
- [ ] All existing tests pass
- [ ] No performance regression

### Compatibility Requirements

- [ ] Legacy flat trait keys maintained during transition
- [ ] Legacy facets still populated for backward compatibility
- [ ] Schema conversion functions work bidirectionally
- [ ] Existing CLI behavior unchanged

### Quality Requirements

- [ ] Code coverage maintained or improved
- [ ] Documentation updated for new structures
- [ ] Error handling robust for malformed data
- [ ] Logging provides useful debugging information

## Dependencies

- **Phase 1**: New trait structures (`NestedTraits`, `AgentTraits`, etc.)
- **Phase 2**: Field registry for testing nested field resolution

## Risk Mitigation

### High-Risk Areas

1. **Macro System Complexity**: Nested object merging is complex
   - **Mitigation**: Extensive testing, gradual rollout, fallback to flat keys

2. **Performance Impact**: Nested object creation/serialization overhead
   - **Mitigation**: Performance testing, benchmarking, optimization if needed

3. **Backward Compatibility**: Breaking existing integrations
   - **Mitigation**: Maintain legacy keys, comprehensive compatibility testing

### Testing Strategy

1. **Unit Tests**: Each detector individually
2. **Integration Tests**: Full detection pipeline
3. **Compatibility Tests**: Legacy conversion roundtrips
4. **Performance Tests**: No regression in detection speed
5. **Manual Tests**: Real-world environment scenarios

## Implementation Timeline

| Task                    | Duration | Dependencies  | Risk Level |
| ----------------------- | -------- | ------------- | ---------- |
| 3.1 Terminal Detector   | 1-2 days | Phase 1       | Medium     |
| 3.2 Agent Detector      | 1 day    | Phase 1       | Low        |
| 3.3 IDE Detector        | 1 day    | Phase 1       | Low        |
| 3.4 CI Detector         | 1-2 days | Phase 1       | Medium     |
| 3.5 Macro Enhancement   | 2-3 days | Tasks 3.1-3.4 | High       |
| 3.6 Engine Update       | 1 day    | Task 3.5      | Low        |
| 3.7 Evidence Updates    | 1 day    | Tasks 3.1-3.4 | Low        |
| 3.8 Integration Testing | 1-2 days | All above     | Medium     |

**Total Estimated Time**: 8-12 days n/

## Rollback Plan

If critical issues are discovered:

1. **Immediate**: Revert to flat trait keys in detectors
2. **Short-term**: Disable nested object merging in macro
3. **Long-term**: Redesign nested structure if fundamental issues found

## Next Steps

After Phase 3 completion:

- **Phase 4**: Update CLI integration to use new nested structure
- **Phase 5**: Remove legacy compatibility code and finalize migration

## Notes

- Maintain extensive logging during development for debugging
- Consider feature flags for gradual rollout if needed
- Document any performance implications discovered during implementation
- Keep detailed notes on any edge cases discovered for future reference
