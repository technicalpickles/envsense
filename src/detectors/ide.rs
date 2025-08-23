use crate::evidence::{EvidenceItem, EvidenceSource};
use crate::schema::{ContextKind, Facets, IdeId};

use super::{Detection, Detector, EnvSnapshot};

pub struct IdeDetector;

impl Detector for IdeDetector {
    fn name(&self) -> &'static str {
        "ide"
    }

    fn detect(&self, snap: &EnvSnapshot) -> Detection {
        let mut det = Detection::default();
        let env = &snap.env;

        if let Some(val) = env.get("CURSOR_TRACE_ID") {
            det.contexts_add.push(ContextKind::Ide);
            det.facets_patch = Some(Facets {
                ide_id: Some(IdeId::Cursor),
                ..Default::default()
            });
            det.evidence.push(EvidenceItem {
                source: EvidenceSource::Env,
                key: "CURSOR_TRACE_ID".into(),
                value: Some(val.clone()),
                weight: 95,
                note: None,
            });
            det.confidence = 95;
            return det;
        }

        if let Some(hook) = env.get("VSCODE_IPC_HOOK_EXTHOST")
            && hook.contains("Code - Insiders")
        {
            det.contexts_add.push(ContextKind::Ide);
            det.facets_patch = Some(Facets {
                ide_id: Some(IdeId::VscodeInsiders),
                ..Default::default()
            });
            det.evidence.push(EvidenceItem {
                source: EvidenceSource::Env,
                key: "VSCODE_IPC_HOOK_EXTHOST".into(),
                value: Some(hook.clone()),
                weight: 90,
                note: None,
            });
            det.confidence = 90;
            return det;
        }

        if let Some(ver) = env.get("TERM_PROGRAM_VERSION")
            && ver.to_lowercase().contains("insider")
        {
            det.contexts_add.push(ContextKind::Ide);
            det.facets_patch = Some(Facets {
                ide_id: Some(IdeId::VscodeInsiders),
                ..Default::default()
            });
            det.evidence.push(EvidenceItem {
                source: EvidenceSource::Env,
                key: "TERM_PROGRAM_VERSION".into(),
                value: Some(ver.clone()),
                weight: 85,
                note: None,
            });
            det.confidence = 85;
            return det;
        }

        if env.get("VSCODE_PID").is_some()
            || env
                .get("TERM_PROGRAM")
                .map(|v| v == "vscode")
                .unwrap_or(false)
        {
            det.contexts_add.push(ContextKind::Ide);
            det.facets_patch = Some(Facets {
                ide_id: Some(IdeId::Vscode),
                ..Default::default()
            });
            det.evidence.push(EvidenceItem {
                source: EvidenceSource::Env,
                key: if env.get("VSCODE_PID").is_some() {
                    "VSCODE_PID".into()
                } else {
                    "TERM_PROGRAM".into()
                },
                value: env
                    .get("VSCODE_PID")
                    .cloned()
                    .or_else(|| env.get("TERM_PROGRAM").cloned()),
                weight: 90,
                note: None,
            });
            det.confidence = 90;
            return det;
        }

        if env.get("CURSOR_CLI").is_some()
            || env
                .get("TERM_PROGRAM")
                .map(|v| v == "cursor")
                .unwrap_or(false)
        {
            det.contexts_add.push(ContextKind::Ide);
            det.facets_patch = Some(Facets {
                ide_id: Some(IdeId::Cursor),
                ..Default::default()
            });
            det.evidence.push(EvidenceItem {
                source: EvidenceSource::Env,
                key: if env.get("CURSOR_CLI").is_some() {
                    "CURSOR_CLI".into()
                } else {
                    "TERM_PROGRAM".into()
                },
                value: env
                    .get("CURSOR_CLI")
                    .cloned()
                    .or_else(|| env.get("TERM_PROGRAM").cloned()),
                weight: 85,
                note: None,
            });
            det.confidence = 85;
            return det;
        }

        det
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detectors::TtyInfo;

    fn run(env: &[(&str, &str)]) -> Detection {
        let snap = EnvSnapshot {
            env: env
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            tty: TtyInfo,
            proc_hint: None,
        };
        IdeDetector.detect(&snap)
    }

    #[test]
    fn detects_vscode() {
        let det = run(&[("VSCODE_PID", "123")]);
        assert!(det.contexts_add.contains(&ContextKind::Ide));
        assert_eq!(det.facets_patch.unwrap().ide_id, Some(IdeId::Vscode));
    }

    #[test]
    fn detects_vscode_insiders_from_version() {
        let det = run(&[("TERM_PROGRAM_VERSION", "1.2.3-insider")]);
        assert_eq!(
            det.facets_patch.unwrap().ide_id,
            Some(IdeId::VscodeInsiders)
        );
    }

    #[test]
    fn detects_cursor_cli() {
        let det = run(&[("CURSOR_CLI", "1")]);
        assert_eq!(det.facets_patch.unwrap().ide_id, Some(IdeId::Cursor));
    }

    #[test]
    fn detects_cursor_trace_id_overrides_vscode() {
        let det = run(&[("CURSOR_TRACE_ID", "xyz"), ("VSCODE_PID", "2")]);
        assert_eq!(det.facets_patch.unwrap().ide_id, Some(IdeId::Cursor));
    }

    #[test]
    fn confidence_prefers_higher() {
        let det = run(&[("CURSOR_CLI", "1"), ("VSCODE_PID", "2")]);
        assert_eq!(det.facets_patch.unwrap().ide_id, Some(IdeId::Vscode));
    }
}
