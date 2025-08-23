use std::collections::BTreeMap;

use crate::evidence::EvidenceItem;
use crate::schema::{ContextKind, Facets, Traits};

#[derive(Default, Clone)]
pub struct TtyInfo;

#[derive(Default, Clone)]
pub struct ProcHint;

pub struct EnvSnapshot {
    pub env: BTreeMap<String, String>,
    pub tty: TtyInfo,
    pub proc_hint: Option<ProcHint>,
}

pub struct Detection {
    pub contexts_add: Vec<ContextKind>,
    pub traits_patch: Option<Traits>,
    pub facets_patch: Option<Facets>,
    pub evidence: Vec<EvidenceItem>,
    pub confidence: u8,
}

impl Default for Detection {
    fn default() -> Self {
        Self {
            contexts_add: Vec::new(),
            traits_patch: None,
            facets_patch: None,
            evidence: Vec::new(),
            confidence: 0,
        }
    }
}

pub trait Detector {
    fn name(&self) -> &'static str;
    fn detect(&self, snap: &EnvSnapshot) -> Detection;
    fn depends_on(&self) -> &'static [&'static str] {
        &[]
    }
}

pub mod agent;
pub mod ci;
pub mod ide;

pub fn registry() -> Vec<Box<dyn Detector>> {
    vec![
        Box::new(agent::AgentDetector),
        Box::new(ci::CiDetector),
        Box::new(ide::IdeDetector),
    ]
}
