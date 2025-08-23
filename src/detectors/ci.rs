use ci_info::{get, types::Vendor};

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
        let info = get();
        if info.ci {
            det.contexts_add.push(ContextKind::Ci);
            let (ci_id, key, weight) = match info.vendor {
                Some(Vendor::GitHubActions) => (CiId::Github, "GITHUB_ACTIONS", 90),
                _ => (CiId::Generic, "CI", 80),
            };
            det.facets_patch = Some(Facets {
                ci_id: Some(ci_id),
                ..Default::default()
            });
            det.evidence.push(EvidenceItem {
                source: EvidenceSource::Env,
                key: key.into(),
                value: snap.env.get(key).cloned(),
                weight,
                note: None,
            });
            det.confidence = weight;
        }
        det
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detectors::TtyInfo;
    use temp_env::with_vars;

    fn run(env: &[(&str, &str)]) -> Detection {
        with_vars(
            env.iter().map(|(k, v)| (*k, Some(*v))).collect::<Vec<_>>(),
            || {
                let snap = EnvSnapshot {
                    env: env
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                    tty: TtyInfo,
                    proc_hint: None,
                };
                CiDetector.detect(&snap)
            },
        )
    }

    #[test]
    fn detects_github_actions() {
        let det = run(&[("GITHUB_ACTIONS", "1")]);
        assert!(det.contexts_add.contains(&ContextKind::Ci));
        assert_eq!(det.facets_patch.unwrap().ci_id, Some(CiId::Github));
    }

    #[test]
    fn detects_generic_ci() {
        let det = run(&[("CI", "true")]);
        assert_eq!(det.facets_patch.unwrap().ci_id, Some(CiId::Generic));
    }
}
