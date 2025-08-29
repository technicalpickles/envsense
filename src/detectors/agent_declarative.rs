use crate::detectors::env_mapping::{get_agent_mappings, get_host_mappings};
use crate::detectors::utils::check_generic_overrides;
use crate::detectors::{Detection, Detector, EnvSnapshot};
use crate::schema::Evidence;
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

                    // Add evidence for this detection
                    for (key, value) in mapping.get_evidence(&snap.env_vars) {
                        let evidence_item = if let Some(val) = value {
                            Evidence::env_var(key, val)
                        } else {
                            Evidence::env_presence(key)
                        };
                        evidence.push(
                            evidence_item
                                .with_supports(vec!["agent".into(), "agent_id".into()])
                                .with_confidence(mapping.confidence),
                        );
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
                                Evidence::env_var(key, val)
                            } else {
                                Evidence::env_presence(key)
                            };
                            evidence.push(
                                evidence_item
                                    .with_supports(vec!["host".into()])
                                    .with_confidence(mapping.confidence),
                            );
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
    use std::collections::HashMap;

    fn create_env_snapshot(env_vars: Vec<(&str, &str)>) -> EnvSnapshot {
        let mut env_map = HashMap::new();
        for (k, v) in env_vars {
            env_map.insert(k.to_string(), v.to_string());
        }

        EnvSnapshot::with_mock_tty(env_map, false, false, false)
    }

    #[test]
    fn detects_cursor_agent() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("CURSOR_AGENT", "1")]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.contains(&"agent".to_string()));
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
    fn detects_replit_host_only() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("REPLIT_USER", "josh")]);

        let detection = detector.detect(&snapshot);

        // Should not detect as agent, but should detect host
        assert!(!detection.contexts_add.contains(&"agent".to_string()));
        assert_eq!(
            detection.facets_patch.get("host").unwrap(),
            &json!("replit")
        );
    }

    #[test]
    fn respects_override_force_human() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot =
            create_env_snapshot(vec![("ENVSENSE_ASSUME_HUMAN", "1"), ("CURSOR_AGENT", "1")]);

        let detection = detector.detect(&snapshot);

        // Should not detect as agent despite CURSOR_AGENT being present
        assert!(!detection.contexts_add.contains(&"agent".to_string()));
        assert!(detection.facets_patch.get("agent_id").is_none());
    }

    #[test]
    fn respects_override_force_agent() {
        let detector = DeclarativeAgentDetector::new();
        let snapshot = create_env_snapshot(vec![("ENVSENSE_AGENT", "custom-agent")]);

        let detection = detector.detect(&snapshot);

        assert!(detection.contexts_add.contains(&"agent".to_string()));
        assert_eq!(
            detection.facets_patch.get("agent_id").unwrap(),
            &json!("custom-agent")
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
        assert_eq!(
            detection.facets_patch.get("agent_id").unwrap(),
            &json!("aider")
        );
        assert_eq!(detection.confidence, 0.8);
    }
}
