//! Runtime prompt presentation and the authored consequences of each choice.

use crate::{Effect, PromptMode, ResolutionPolicy, Turn};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Prompt {
    pub id: String,
    pub respondent: String,
    pub mode: PromptMode,
    #[serde(default = "one_usize")]
    pub min_choices: usize,
    #[serde(default = "one_usize")]
    pub max_choices: usize,
    #[serde(default = "first_response")]
    pub resolution: ResolutionPolicy,
    pub choices: Vec<Choice>,
}
fn one_usize() -> usize {
    1
}
fn first_response() -> ResolutionPolicy {
    ResolutionPolicy::FirstResponse
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub effects: Vec<Effect>,
    /// Authored transcript turns appended after this choice wins resolution.
    #[serde(default)]
    pub result_turns: Vec<Turn>,
}
