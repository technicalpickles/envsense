use crate::ci::CiFacet;
use crate::detectors::{Detector, EnvSnapshot};
use crate::schema::{Contexts, EnvSense, Facets, SCHEMA_VERSION, Traits};
use crate::traits::terminal::ColorLevel;
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
        Self::set_context_bool(&mut result.contexts, "agent", &all_contexts);
        Self::set_context_bool(&mut result.contexts, "ide", &all_contexts);
        Self::set_context_bool(&mut result.contexts, "ci", &all_contexts);
        Self::set_context_bool(&mut result.contexts, "container", &all_contexts); // TODO: Implement container detector
        Self::set_context_bool(&mut result.contexts, "remote", &all_contexts);

        // Set facet IDs
        Self::set_facet_id(&mut result.facets.agent_id, "agent_id", &all_facets);
        Self::set_facet_id(&mut result.facets.ide_id, "ide_id", &all_facets);
        Self::set_facet_id(&mut result.facets.ci_id, "ci_id", &all_facets);

        // Set boolean traits
        Self::set_trait_bool(&mut result.traits.is_interactive, "is_interactive", &all_traits);
        Self::set_trait_bool(&mut result.traits.is_tty_stdin, "is_tty_stdin", &all_traits);
        Self::set_trait_bool(&mut result.traits.is_tty_stdout, "is_tty_stdout", &all_traits);
        Self::set_trait_bool(&mut result.traits.is_tty_stderr, "is_tty_stderr", &all_traits);
        Self::set_trait_bool(&mut result.traits.is_piped_stdin, "is_piped_stdin", &all_traits);
        Self::set_trait_bool(&mut result.traits.is_piped_stdout, "is_piped_stdout", &all_traits);
        Self::set_trait_bool(&mut result.traits.supports_hyperlinks, "supports_hyperlinks", &all_traits);

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
        if let Some(ci_facet_value) = all_facets.get("ci")
            && let Ok(ci_facet) = serde_json::from_value::<CiFacet>(ci_facet_value.clone()) {
                result.facets.ci = ci_facet;
            }

        result
    }

    fn set_context_bool(contexts: &mut Contexts, context_name: &str, all_contexts: &std::collections::HashSet<String>) {
        match context_name {
            "agent" => contexts.agent = all_contexts.contains("agent"),
            "ide" => contexts.ide = all_contexts.contains("ide"),
            "ci" => contexts.ci = all_contexts.contains("ci"),
            "container" => contexts.container = all_contexts.contains("container"),
            "remote" => contexts.remote = all_contexts.contains("remote"),
            _ => {}
        }
    }

    fn set_facet_id(facet_id: &mut Option<String>, facet_name: &str, all_facets: &HashMap<String, serde_json::Value>) {
        if let Some(value) = all_facets.get(facet_name).and_then(|v| v.as_str()) {
            *facet_id = Some(value.to_string());
        }
    }

    fn set_trait_bool(trait_field: &mut bool, trait_name: &str, all_traits: &HashMap<String, serde_json::Value>) {
        if let Some(value) = all_traits.get(trait_name).and_then(|v| v.as_bool()) {
            *trait_field = value;
        }
    }
}

impl Default for DetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}
