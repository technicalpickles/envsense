use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceSource {
    Env,
    Tty,
    Proc,
    Fs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub source: EvidenceSource,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default)]
    pub supports: Vec<String>,
    pub confidence: f32,
}

impl EvidenceItem {
    pub fn env_var(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            source: EvidenceSource::Env,
            key: key.into(),
            value: Some(value.into()),
            supports: Vec::new(),
            confidence: 0.9,
        }
    }
    
    pub fn env_presence(key: impl Into<String>) -> Self {
        Self {
            source: EvidenceSource::Env,
            key: key.into(),
            value: None,
            supports: Vec::new(),
            confidence: 0.9,
        }
    }
    
    pub fn tty_trait(key: impl Into<String>, is_tty: bool) -> Self {
        Self {
            source: EvidenceSource::Tty,
            key: key.into(),
            value: Some(is_tty.to_string()),
            supports: Vec::new(),
            confidence: 1.0,
        }
    }
    
    pub fn with_supports(mut self, supports: Vec<String>) -> Self {
        self.supports = supports;
        self
    }
    
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}