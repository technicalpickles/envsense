use crate::traits::{AgentTraits, CiTraits, IdeTraits, NestedTraits, StreamInfo, TerminalTraits};
use envsense_macros::{Detection, DetectionMerger, DetectionMergerDerive};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::evidence::Evidence;
use super::legacy::{Contexts, Facets, LegacyEnvSense, Traits};

/// New nested schema structure using the nested traits system
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, DetectionMergerDerive)]
pub struct NewEnvSense {
    pub contexts: Vec<String>, // Simplified from Contexts struct
    pub traits: NestedTraits,  // New nested structure
    pub evidence: Vec<Evidence>,
    pub version: String,
}

impl Default for NewEnvSense {
    fn default() -> Self {
        Self {
            contexts: Vec::new(),
            traits: NestedTraits::default(),
            evidence: Vec::new(),
            version: super::SCHEMA_VERSION.to_string(),
        }
    }
}

impl NewEnvSense {
    /// Convert new schema to legacy format for backward compatibility
    pub fn to_legacy(&self) -> LegacyEnvSense {
        LegacyEnvSense {
            contexts: Contexts {
                agent: self.contexts.contains(&"agent".to_string()),
                ide: self.contexts.contains(&"ide".to_string()),
                ci: self.contexts.contains(&"ci".to_string()),
                container: self.contexts.contains(&"container".to_string()),
                remote: self.contexts.contains(&"remote".to_string()),
            },
            facets: Facets {
                agent_id: self.traits.agent.id.clone(),
                ide_id: self.traits.ide.id.clone(),
                ci_id: self.traits.ci.id.clone(),
                container_id: None, // New schema doesn't have container info yet
                host: None,         // New schema doesn't have host info yet
            },
            traits: Traits {
                is_interactive: self.traits.terminal.interactive,
                is_tty_stdin: self.traits.terminal.stdin.tty,
                is_tty_stdout: self.traits.terminal.stdout.tty,
                is_tty_stderr: self.traits.terminal.stderr.tty,
                is_piped_stdin: self.traits.terminal.stdin.piped,
                is_piped_stdout: self.traits.terminal.stdout.piped,
                color_level: self.traits.terminal.color_level.clone(),
                supports_hyperlinks: self.traits.terminal.supports_hyperlinks,
                is_ci: if self.traits.ci.id.is_some() {
                    Some(true)
                } else {
                    None
                },
                ci_vendor: self.traits.ci.vendor.clone(),
                ci_name: self.traits.ci.name.clone(),
                is_pr: self.traits.ci.is_pr,
                ci_pr: self.traits.ci.is_pr,
                branch: self.traits.ci.branch.clone(),
            },
            evidence: self.evidence.clone(),
            version: super::LEGACY_SCHEMA_VERSION.to_string(),
        }
    }

    /// Convert legacy schema to new format
    pub fn from_legacy(legacy: &LegacyEnvSense) -> Self {
        let mut contexts = Vec::new();
        if legacy.contexts.agent {
            contexts.push("agent".to_string());
        }
        if legacy.contexts.ide {
            contexts.push("ide".to_string());
        }
        if legacy.contexts.ci {
            contexts.push("ci".to_string());
        }
        if legacy.contexts.container {
            contexts.push("container".to_string());
        }
        if legacy.contexts.remote {
            contexts.push("remote".to_string());
        }

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
                    stdin: StreamInfo {
                        tty: legacy.traits.is_tty_stdin,
                        piped: legacy.traits.is_piped_stdin,
                    },
                    stdout: StreamInfo {
                        tty: legacy.traits.is_tty_stdout,
                        piped: legacy.traits.is_piped_stdout,
                    },
                    stderr: StreamInfo {
                        tty: legacy.traits.is_tty_stderr,
                        piped: false, // Legacy doesn't have stderr piped info
                    },
                    supports_hyperlinks: legacy.traits.supports_hyperlinks,
                },
                ci: CiTraits {
                    id: legacy.facets.ci_id.clone(),
                    vendor: legacy.traits.ci_vendor.clone(),
                    name: legacy.traits.ci_name.clone(),
                    is_pr: legacy.traits.is_pr.or(legacy.traits.ci_pr),
                    branch: legacy.traits.branch.clone(),
                },
            },
            evidence: legacy.evidence.clone(),
            version: super::SCHEMA_VERSION.to_string(),
        }
    }
}
