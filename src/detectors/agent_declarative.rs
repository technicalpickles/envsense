use crate::detectors::env_mapping::{get_agent_mappings, get_host_mappings};
use crate::detectors::utils::check_generic_overrides;
use crate::detectors::{Detection, Detector, EnvSnapshot};
use crate::schema::Evidence;
use crate::traits::AgentTraits;
use serde_json::json;

pub struct DeclarativeAgentDetector;

impl DeclarativeAgentDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect agent and host environments using declarative mappings
    fn detect_environments(
        &self,
        snap: &EnvSnapshot,
    ) -> (Option<String>, Option<String>, f32, Vec<Evidence>) {
        let mut agent_id = None;
        let mut host_id = None;
        let mut confidence = 0.0;
        let mut evidence = Vec::new();

        // Check for overrides first
        let mut skip_host_detection = false;
        if let Some(override_result) = check_generic_overrides(snap, "agent") {
            let (override_agent_id, override_confidence, override_evidence) = override_result;

            // Skip host detection only if agent_id is None (assume human override)
            skip_host_detection = override_agent_id.is_none();

            agent_id = override_agent_id;
            confidence = override_confidence;
            evidence = override_evidence;
        } else {
            // Use declarative mappings for agent detection
            let agent_mappings = get_agent_mappings();

            // Find the highest confidence matching agent
            for mapping in &agent_mappings {
                // Only consider mappings that add agent context
                if mapping.contexts.contains(&"agent".to_string())
                    && mapping.matches(&snap.env_vars)
                    && mapping.confidence > confidence
                {
                    agent_id = Some(mapping.id.clone());
                    confidence = mapping.confidence;

                    // Add evidence for this detection using helper methods
                    for (key, value) in mapping.get_evidence(&snap.env_vars) {
                        let evidence_item = if let Some(val) = value {
                            // Check if this mapping also provides host information
                            if mapping.facets.contains_key("host") {
                                Evidence::agent_with_host_detection(key, val)
                            } else {
                                Evidence::agent_detection(key, val)
                            }
                        } else {
                            Evidence::env_presence(key).with_supports(vec!["agent.id".into()])
                        };
                        evidence.push(evidence_item.with_confidence(mapping.confidence));
                    }

                    // Add any facets from the mapping
                    if let Some(host) = mapping.facets.get("host") {
                        host_id = Some(host.clone());
                    }

                    break; // Take the first (highest confidence) match
                }
            }
        }

        // Detect host if not already set and not skipping host detection
        if host_id.is_none() && !skip_host_detection {
            // First check if any agent mappings also set host
            let agent_mappings = get_agent_mappings();
            for mapping in &agent_mappings {
                if mapping.matches(&snap.env_vars)
                    && let Some(host) = mapping.facets.get("host")
                {
                    host_id = Some(host.clone());
                    break;
                }
            }

            // If no host from agent mappings, check dedicated host mappings
            if host_id.is_none() {
                let host_mappings = get_host_mappings();

                for mapping in &host_mappings {
                    if mapping.matches(&snap.env_vars)
                        && let Some(host) = mapping.facets.get("host")
                    {
                        host_id = Some(host.clone());

                        // Add evidence for host detection
                        for (key, value) in mapping.get_evidence(&snap.env_vars) {
                            let evidence_item = if let Some(val) = value {
                                Evidence::env_var(key, val).with_supports(vec!["host".into()])
                            } else {
                                Evidence::env_presence(key).with_supports(vec!["host".into()])
                            };
                            evidence.push(evidence_item.with_confidence(mapping.confidence));
                        }
                        break;
                    }
                }
            }

            // Default host if none detected
            if host_id.is_none() {
                host_id = Some("unknown".to_string());
            }
        }

        (agent_id, host_id, confidence, evidence)
    }
}

impl Detector for DeclarativeAgentDetector {
    fn name(&self) -> &'static str {
        "declarative_agent"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut detection = Detection::default();

        let (agent_id, host_id, confidence, evidence) = self.detect_environments(snap);

        // Add agent detection
        if let Some(agent) = agent_id {
            detection.contexts_add.push("agent".to_string());
            detection.confidence = confidence;

            // Create nested AgentTraits object
            let agent_traits = AgentTraits {
                id: Some(agent.clone()),
            };

            // Insert as nested object under "agent" key
            detection.traits_patch.insert(
                "agent".to_string(),
                serde_json::to_value(agent_traits).unwrap(),
            );

            // Keep legacy facets for backward compatibility
            detection
                .facets_patch
                .insert("agent_id".to_string(), json!(agent));
        }

        // Add host detection
        if let Some(host) = host_id {
            detection
                .facets_patch
                .insert("host".to_string(), json!(host));
        }

        // Add all evidence
        detection.evidence = evidence;

        detection
    }
}

impl Default for DeclarativeAgentDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::detectors::test_utils::create_env_snapshot;

    // =============================================================================
    // Basic Agent Detection Tests
    // =============================================================================

    #[test]
    fn detects_cursor_agent() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("CURSOR_AGENT", "1")]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.contains(&"agent".to_string()));

        // Verify nested AgentTraits object
        let agent_traits_value = detection.traits_patch.get("agent").unwrap();
        let agent_traits: AgentTraits = serde_json::from_value(agent_traits_value.clone()).unwrap();
        assert_eq!(agent_traits.id, Some("cursor".to_string()));

        // Verify legacy facet is maintained
        assert_eq!(
            detection.facets_patch.get("agent_id").unwrap(),
            &json!("cursor")
        );
        assert_eq!(detection.confidence, 1.0);
    }

    #[test]
    fn detects_replit_agent() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("REPL_ID", "abc123")]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.contains(&"agent".to_string()));

        // Verify nested AgentTraits object
        let agent_traits_value = detection.traits_patch.get("agent").unwrap();
        let agent_traits: AgentTraits = serde_json::from_value(agent_traits_value.clone()).unwrap();
        assert_eq!(agent_traits.id, Some("replit-agent".to_string()));

        // Verify legacy facets are maintained
        assert_eq!(
            detection.facets_patch.get("agent_id").unwrap(),
            &json!("replit-agent")
        );
        assert_eq!(
            detection.facets_patch.get("host").unwrap(),
            &json!("replit")
        );
        assert_eq!(detection.confidence, 1.0);
    }

    #[test]
    fn detects_openhands_with_prefix() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![
            ("SANDBOX_VOLUMES", "/tmp"),
            ("SANDBOX_RUNTIME_CONTAINER_IMAGE", "alpine"),
        ]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.contains(&"agent".to_string()));

        // Verify nested AgentTraits object
        let agent_traits_value = detection.traits_patch.get("agent").unwrap();
        let agent_traits: AgentTraits = serde_json::from_value(agent_traits_value.clone()).unwrap();
        assert_eq!(agent_traits.id, Some("openhands".to_string()));

        // Verify legacy facet is maintained
        assert_eq!(
            detection.facets_patch.get("agent_id").unwrap(),
            &json!("openhands")
        );
        assert_eq!(detection.confidence, 0.8);
    }

    #[test]
    fn detects_aider() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("AIDER_MODEL", "gpt-4o-mini")]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.contains(&"agent".to_string()));

        // Verify nested AgentTraits object
        let agent_traits_value = detection.traits_patch.get("agent").unwrap();
        let agent_traits: AgentTraits = serde_json::from_value(agent_traits_value.clone()).unwrap();
        assert_eq!(agent_traits.id, Some("aider".to_string()));

        // Verify legacy facet is maintained
        assert_eq!(
            detection.facets_patch.get("agent_id").unwrap(),
            &json!("aider")
        );
        assert_eq!(detection.confidence, 0.8);
    }

    // =============================================================================
    // Override Scenario Tests
    // =============================================================================

    #[test]
    fn respects_override_force_human() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot =
            create_env_snapshot(vec![("ENVSENSE_ASSUME_HUMAN", "1"), ("CURSOR_AGENT", "1")]);

        let detection = detector.detect(&snapshot);

        // Should not detect as agent despite CURSOR_AGENT being present
        assert!(!detection.contexts_add.contains(&"agent".to_string()));

        // Should not have agent traits object
        assert!(detection.traits_patch.get("agent").is_none());

        // Should not have legacy facet either
        assert!(detection.facets_patch.get("agent_id").is_none());
    }

    #[test]
    fn respects_override_force_agent() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("ENVSENSE_AGENT", "custom-agent")]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.contains(&"agent".to_string()));

        // Verify nested AgentTraits object
        let agent_traits_value = detection.traits_patch.get("agent").unwrap();
        let agent_traits: AgentTraits = serde_json::from_value(agent_traits_value.clone()).unwrap();
        assert_eq!(agent_traits.id, Some("custom-agent".to_string()));

        // Verify legacy facet is maintained
        assert_eq!(
            detection.facets_patch.get("agent_id").unwrap(),
            &json!("custom-agent")
        );
        assert_eq!(detection.confidence, 1.0);
    }

    // =============================================================================
    // Host-Only Detection Tests
    // =============================================================================

    #[test]
    fn detects_replit_host_only() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("REPLIT_USER", "josh")]);

        let detection = detector.detect(&snapshot);

        // Should not detect as agent, but should detect host
        assert!(!detection.contexts_add.contains(&"agent".to_string()));

        // Should not have agent traits object since no agent detected
        assert!(detection.traits_patch.get("agent").is_none());

        // Should still detect host
        assert_eq!(
            detection.facets_patch.get("host").unwrap(),
            &json!("replit")
        );
    }

    // =============================================================================
    // Nested Structure Validation Tests
    // =============================================================================

    #[test]
    fn test_nested_agent_traits_json_structure() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("CURSOR_AGENT", "1")]);

        let detection = detector.detect(&snapshot);

        // Verify the JSON structure of the nested AgentTraits object
        let agent_traits_value = detection.traits_patch.get("agent").unwrap();
        let json_string = serde_json::to_string_pretty(agent_traits_value).unwrap();

        // Should contain the nested structure
        assert!(json_string.contains("\"id\""));
        assert!(json_string.contains("\"cursor\""));

        // Verify it can be deserialized back to AgentTraits
        let agent_traits: AgentTraits = serde_json::from_value(agent_traits_value.clone()).unwrap();
        assert_eq!(agent_traits.id, Some("cursor".to_string()));

        // Verify the complete traits_patch structure
        assert_eq!(detection.traits_patch.len(), 1);
        assert!(detection.traits_patch.contains_key("agent"));
    }

    // =============================================================================
    // Evidence Field Path Tests
    // =============================================================================

    #[test]
    fn test_evidence_uses_correct_field_paths() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("CURSOR_AGENT", "1")]);

        let detection = detector.detect(&snapshot);

        // Verify evidence exists
        assert!(!detection.evidence.is_empty());

        // Find agent-related evidence
        let agent_evidence: Vec<_> = detection
            .evidence
            .iter()
            .filter(|e| e.supports.contains(&"agent.id".to_string()))
            .collect();

        assert!(
            !agent_evidence.is_empty(),
            "Should have evidence supporting agent.id"
        );

        // Verify the evidence uses the correct nested field path
        for evidence in agent_evidence {
            assert!(
                evidence.supports.contains(&"agent.id".to_string()),
                "Evidence should support 'agent.id' field path, got: {:?}",
                evidence.supports
            );

            // Should not use legacy field paths
            assert!(
                !evidence.supports.contains(&"agent_id".to_string()),
                "Evidence should not use legacy 'agent_id' field path"
            );
            assert!(
                !evidence.supports.contains(&"agent".to_string()) || evidence.supports.len() > 1,
                "Evidence should not use bare 'agent' context as field path"
            );
        }
    }

    #[test]
    fn test_evidence_field_paths_with_override() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("ENVSENSE_AGENT", "test-agent")]);

        let detection = detector.detect(&snapshot);

        // Verify evidence exists for override scenario
        assert!(
            !detection.evidence.is_empty(),
            "Should have some evidence for override"
        );

        // Verify the agent detection actually worked
        assert!(detection.contexts_add.contains(&"agent".to_string()));
        assert!(detection.traits_patch.contains_key("agent"));

        // NOTE: Override evidence now uses nested field paths after utils::check_generic_overrides was updated
        let override_evidence: Vec<_> = detection
            .evidence
            .iter()
            .filter(|e| e.key == "ENVSENSE_AGENT")
            .collect();

        assert!(
            !override_evidence.is_empty(),
            "Should have override evidence"
        );

        // Override evidence now uses nested field paths after utils::check_generic_overrides was updated
        for evidence in override_evidence {
            assert!(
                evidence.supports.contains(&"agent.id".to_string()),
                "Override evidence should use nested field paths: {:?}",
                evidence.supports
            );
        }
    }

    // =============================================================================
    // Edge Case and Error Handling Tests
    // =============================================================================

    #[test]
    fn test_no_agent_detection_produces_empty_traits_patch() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("SOME_OTHER_VAR", "value")]);

        let detection = detector.detect(&snapshot);

        // Should not detect as agent
        assert!(!detection.contexts_add.contains(&"agent".to_string()));

        // Should not have agent traits object
        assert!(detection.traits_patch.get("agent").is_none());

        // Should not have agent_id in facets
        assert!(detection.facets_patch.get("agent_id").is_none());

        // Should have no evidence supporting agent.id
        let agent_evidence: Vec<_> = detection
            .evidence
            .iter()
            .filter(|e| e.supports.contains(&"agent.id".to_string()))
            .collect();
        assert!(agent_evidence.is_empty());
    }

    #[test]
    fn test_agent_traits_serialization_consistency() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("CURSOR_AGENT", "1")]);

        let detection = detector.detect(&snapshot);

        // Get the nested AgentTraits object
        let agent_traits_value = detection.traits_patch.get("agent").unwrap();
        let agent_traits: AgentTraits = serde_json::from_value(agent_traits_value.clone()).unwrap();

        // Serialize it back to JSON
        let reserialized = serde_json::to_value(&agent_traits).unwrap();

        // Should be identical to original
        assert_eq!(agent_traits_value, &reserialized);

        // Should maintain the same structure when pretty-printed
        let pretty1 = serde_json::to_string_pretty(agent_traits_value).unwrap();
        let pretty2 = serde_json::to_string_pretty(&reserialized).unwrap();
        assert_eq!(pretty1, pretty2);
    }

    #[test]
    fn test_traits_patch_structure_validation() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("CURSOR_AGENT", "1")]);

        let detection = detector.detect(&snapshot);

        // Verify traits_patch has exactly one entry for agent detection
        assert_eq!(detection.traits_patch.len(), 1);

        // Verify the key is exactly "agent"
        assert!(detection.traits_patch.contains_key("agent"));

        // Verify the value is an object (not a string or other type)
        let agent_value = detection.traits_patch.get("agent").unwrap();
        assert!(
            agent_value.is_object(),
            "Agent traits should be an object, got: {}",
            agent_value
        );

        // Verify the object has the expected structure
        let agent_obj = agent_value.as_object().unwrap();
        assert!(
            agent_obj.contains_key("id"),
            "Agent object should have 'id' field"
        );
        assert_eq!(
            agent_obj.len(),
            1,
            "Agent object should have exactly one field"
        );

        // Verify the id value
        let id_value = agent_obj.get("id").unwrap();
        assert!(id_value.is_string(), "Agent id should be a string");
        assert_eq!(id_value.as_str().unwrap(), "cursor");
    }

    #[test]
    fn test_multiple_environment_variables_same_agent() {
        let detector = DeclarativeAgentDetector::new();
        // Some agents might have multiple environment variables
        let snapshot = create_env_snapshot(vec![
            ("CURSOR_AGENT", "1"),
            ("CURSOR_VERSION", "0.1.0"), // Additional env var that shouldn't affect detection
        ]);

        let detection = detector.detect(&snapshot);

        // Should still detect cursor agent correctly
        assert!(detection.contexts_add.contains(&"agent".to_string()));

        let agent_traits_value = detection.traits_patch.get("agent").unwrap();
        let agent_traits: AgentTraits = serde_json::from_value(agent_traits_value.clone()).unwrap();
        assert_eq!(agent_traits.id, Some("cursor".to_string()));

        // Should still have exactly one traits_patch entry
        assert_eq!(detection.traits_patch.len(), 1);
    }

    #[test]
    fn test_confidence_levels_preserved_with_nested_structure() {
        // Test different confidence levels from different agents
        let test_cases = vec![
            (vec![("CURSOR_AGENT", "1")], 1.0),
            (vec![("AIDER_MODEL", "gpt-4")], 0.8),
            (
                vec![
                    ("SANDBOX_VOLUMES", "/tmp"),
                    ("SANDBOX_RUNTIME_CONTAINER_IMAGE", "alpine"),
                ],
                0.8,
            ),
        ];

        let detector = DeclarativeAgentDetector::new();

        for (env_vars, expected_confidence) in test_cases {
            let snapshot = create_env_snapshot(env_vars);
            let detection = detector.detect(&snapshot);

            assert_eq!(detection.confidence, expected_confidence);

            // Verify nested structure is still created correctly
            if detection.contexts_add.contains(&"agent".to_string()) {
                let agent_traits_value = detection.traits_patch.get("agent").unwrap();
                let agent_traits: AgentTraits =
                    serde_json::from_value(agent_traits_value.clone()).unwrap();
                assert!(agent_traits.id.is_some());
            }
        }
    }

    // =============================================================================
    // Performance Tests
    // =============================================================================

    #[test]
    fn test_nested_structure_performance() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("CURSOR_AGENT", "1")]);

        // Measure performance of nested object creation
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let detection = detector.detect(&snapshot);
            // Verify the nested structure is created correctly each time
            let agent_traits_value = detection.traits_patch.get("agent").unwrap();
            let _agent_traits: AgentTraits =
                serde_json::from_value(agent_traits_value.clone()).unwrap();
        }
        let duration = start.elapsed();

        // Should complete 1000 iterations quickly (within reasonable time)
        assert!(
            duration.as_millis() < 1000,
            "Nested object creation took too long: {:?}ms for 1000 iterations",
            duration.as_millis()
        );
    }
}
