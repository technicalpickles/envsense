use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::evidence::EvidenceItem;
use crate::traits::terminal::ColorLevel;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ContextKind {
    Agent,
    Ide,
    Ci,
    Container,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Traits {
    pub is_interactive: bool,
    pub color_level: ColorLevel,
    pub supports_hyperlinks: bool,
    pub is_piped_stdin: bool,
    pub is_piped_stdout: bool,
    pub is_tty_stdin: bool,
    pub is_tty_stdout: bool,
    pub is_tty_stderr: bool,
}

impl Default for Traits {
    fn default() -> Self {
        Self {
            is_interactive: false,
            color_level: ColorLevel::None,
            supports_hyperlinks: false,
            is_piped_stdin: false,
            is_piped_stdout: false,
            is_tty_stdin: false,
            is_tty_stdout: false,
            is_tty_stderr: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentId {
    Cursor,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IdeId {
    Vscode,
    VscodeInsiders,
    Cursor,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CiId {
    Github,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct Facets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<AgentId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ide_id: Option<IdeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_id: Option<CiId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Meta {
    pub schema_version: String,
    pub rules_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Report {
    pub contexts: BTreeSet<ContextKind>,
    pub traits: Traits,
    pub facets: Facets,
    pub meta: Meta,
    #[serde(default)]
    pub evidence: Vec<EvidenceItem>,
}
