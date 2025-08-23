use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    Env,
    TtyProbe,
    ProcAncestry,
    Filesystem,
    UserOverride,
    Heuristic,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct EvidenceItem {
    pub source: EvidenceSource,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    pub weight: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}
