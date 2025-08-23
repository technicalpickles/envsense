use std::collections::{BTreeMap, BTreeSet};

use crate::detectors::{self, EnvSnapshot};
use crate::schema::{Facets, Meta, Report, Traits};
use crate::traits::terminal::TerminalTraits;

const SCHEMA_VERSION: &str = "0.1.0";

pub fn detect() -> Report {
    let snap = EnvSnapshot {
        env: std::env::vars().collect::<BTreeMap<_, _>>(),
        tty: detectors::TtyInfo::default(),
        proc_hint: None,
    };
    let mut report = Report {
        contexts: BTreeSet::new(),
        traits: Traits::from(TerminalTraits::detect()),
        facets: Facets::default(),
        meta: Meta {
            schema_version: SCHEMA_VERSION.to_string(),
            rules_version: String::new(),
        },
        evidence: Vec::new(),
    };
    let mut agent_conf = 0u8;
    let mut ide_conf = 0u8;
    let mut ci_conf = 0u8;

    for det in detectors::registry() {
        let d = det.detect(&snap);
        for c in d.contexts_add {
            report.contexts.insert(c);
        }
        if let Some(p) = d.facets_patch {
            if let Some(id) = p.agent_id {
                if d.confidence >= agent_conf {
                    report.facets.agent_id = Some(id);
                    agent_conf = d.confidence;
                }
            }
            if let Some(id) = p.ide_id {
                if d.confidence >= ide_conf {
                    report.facets.ide_id = Some(id);
                    ide_conf = d.confidence;
                }
            }
            if let Some(id) = p.ci_id {
                if d.confidence >= ci_conf {
                    report.facets.ci_id = Some(id);
                    ci_conf = d.confidence;
                }
            }
        }
        if let Some(t) = d.traits_patch {
            report.traits = t;
        }
        report.evidence.extend(d.evidence);
    }

    report
}

impl From<TerminalTraits> for Traits {
    fn from(t: TerminalTraits) -> Self {
        Self {
            is_interactive: t.is_interactive,
            color_level: t.color_level,
            supports_hyperlinks: t.supports_hyperlinks,
            is_piped_stdin: !t.is_tty_stdin,
            is_piped_stdout: !t.is_tty_stdout,
            is_tty_stdin: t.is_tty_stdin,
            is_tty_stdout: t.is_tty_stdout,
            is_tty_stderr: t.is_tty_stderr,
        }
    }
}
