use crate::detectors::{Detector, EnvSnapshot};
use crate::schema::{EnvSense, Contexts, Facets, Traits, SCHEMA_VERSION};
use crate::traits::terminal::ColorLevel;
use crate::ci::CiFacet;
use std::collections::HashMap;

pub struct DetectionEngine {
    detectors: Vec<Box<dyn Detector>>,
}

impl DetectionEngine {
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }
    
    pub fn register<D: Detector + 'static>(mut self, detector: D) -> Self {
        self.detectors.push(Box::new(detector));
        self
    }
    
    pub fn detect(&self) -> EnvSense {
        let snapshot = EnvSnapshot::current();
        self.detect_from_snapshot(&snapshot)
    }
    
    pub fn detect_from_snapshot(&self, snapshot: &EnvSnapshot) -> EnvSense {
        let mut result = EnvSense {
            contexts: Contexts::default(),
            facets: Facets::default(),
            traits: Traits::default(),
            evidence: Vec::new(),
            version: SCHEMA_VERSION.to_string(),
            rules_version: String::new(),
        };
        
        let mut all_contexts = std::collections::HashSet::new();
        let mut all_traits: HashMap<String, serde_json::Value> = HashMap::new();
        let mut all_facets: HashMap<String, serde_json::Value> = HashMap::new();
        
        for detector in &self.detectors {
            let detection = detector.detect(snapshot);
            
            for context in detection.contexts_add {
                all_contexts.insert(context);
            }
            
            all_traits.extend(detection.traits_patch);
            all_facets.extend(detection.facets_patch);
            
            result.evidence.extend(detection.evidence);
        }
        
        // Set boolean contexts for internal consistency
        result.contexts.agent = all_contexts.contains("agent");
        result.contexts.ide = all_contexts.contains("ide");
        result.contexts.ci = all_contexts.contains("ci");
        result.contexts.container = all_contexts.contains("container");
        result.contexts.remote = all_contexts.contains("remote");
        
        if let Some(agent_id) = all_facets.get("agent_id").and_then(|v| v.as_str()) {
            result.facets.agent_id = Some(agent_id.to_string());
        }
        
        if let Some(ide_id) = all_facets.get("ide_id").and_then(|v| v.as_str()) {
            result.facets.ide_id = Some(ide_id.to_string());
        }
        
        if let Some(ci_id) = all_facets.get("ci_id").and_then(|v| v.as_str()) {
            result.facets.ci_id = Some(ci_id.to_string());
        }
        
        if let Some(container_id) = all_facets.get("container_id").and_then(|v| v.as_str()) {
            result.facets.container_id = Some(container_id.to_string());
        }
        
        if let Some(is_interactive) = all_traits.get("is_interactive").and_then(|v| v.as_bool()) {
            result.traits.is_interactive = is_interactive;
        }
        
        if let Some(is_tty_stdin) = all_traits.get("is_tty_stdin").and_then(|v| v.as_bool()) {
            result.traits.is_tty_stdin = is_tty_stdin;
        }
        
        if let Some(is_tty_stdout) = all_traits.get("is_tty_stdout").and_then(|v| v.as_bool()) {
            result.traits.is_tty_stdout = is_tty_stdout;
        }
        
        if let Some(is_tty_stderr) = all_traits.get("is_tty_stderr").and_then(|v| v.as_bool()) {
            result.traits.is_tty_stderr = is_tty_stderr;
        }
        
        if let Some(is_piped_stdin) = all_traits.get("is_piped_stdin").and_then(|v| v.as_bool()) {
            result.traits.is_piped_stdin = is_piped_stdin;
        }
        
        if let Some(is_piped_stdout) = all_traits.get("is_piped_stdout").and_then(|v| v.as_bool()) {
            result.traits.is_piped_stdout = is_piped_stdout;
        }
        
        if let Some(supports_hyperlinks) = all_traits.get("supports_hyperlinks").and_then(|v| v.as_bool()) {
            result.traits.supports_hyperlinks = supports_hyperlinks;
        }
        
        // Handle color level enum
        if let Some(color_level_str) = all_traits.get("color_level").and_then(|v| v.as_str()) {
            result.traits.color_level = match color_level_str {
                "none" => ColorLevel::None,
                "ansi16" => ColorLevel::Ansi16,
                "ansi256" => ColorLevel::Ansi256,
                "truecolor" => ColorLevel::Truecolor,
                _ => ColorLevel::None,
            };
        }
        
        // Handle CI facet
        if let Some(ci_facet_value) = all_facets.get("ci") {
            if let Ok(ci_facet) = serde_json::from_value::<CiFacet>(ci_facet_value.clone()) {
                result.facets.ci = ci_facet;
            }
        }
        
        result
    }
}

impl Default for DetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}