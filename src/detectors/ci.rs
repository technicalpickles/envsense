use crate::evidence::{EvidenceItem, EvidenceSource};
use crate::schema::{CiId, ContextKind, Facets};

use super::{Detection, Detector, EnvSnapshot};

pub struct CiDetector;

impl Detector for CiDetector {
    fn name(&self) -> &'static str {
        "ci"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut det = Detection::default();
        if snap.env.get("GITHUB_ACTIONS").is_some() {
            det.contexts_add.push(ContextKind::Ci);
            det.facets_patch = Some(Facets {
                ci_id: Some(CiId::Github),
                ..Default::default()
            });
            det.evidence.push(EvidenceItem {
                source: EvidenceSource::Env,
                key: "GITHUB_ACTIONS".into(),
                value: None,
                weight: 90,
                note: None,
            });
            det.confidence = 90;
        }
        det
    }
}
