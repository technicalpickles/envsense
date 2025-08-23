use crate::evidence::{EvidenceItem, EvidenceSource};
use crate::schema::{AgentId, ContextKind, Facets};

use super::{Detection, Detector, EnvSnapshot};

pub struct AgentDetector;

impl Detector for AgentDetector {
    fn name(&self) -> &'static str {
        "agent"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut det = Detection::default();
        if let Some(val) = snap.env.get("CURSOR_AGENT") {
            det.contexts_add.push(ContextKind::Agent);
            det.facets_patch = Some(Facets {
                agent_id: Some(AgentId::Cursor),
                ..Default::default()
            });
            det.evidence.push(EvidenceItem {
                source: EvidenceSource::Env,
                key: "CURSOR_AGENT".into(),
                value: Some(val.clone()),
                weight: 80,
                note: None,
            });
            det.confidence = 80;
        }
        det
    }
}
