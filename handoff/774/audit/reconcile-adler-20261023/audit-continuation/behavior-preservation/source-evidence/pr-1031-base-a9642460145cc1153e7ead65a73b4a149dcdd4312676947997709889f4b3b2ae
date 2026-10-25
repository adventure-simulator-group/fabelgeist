//! Canonical authored and persisted dialogue policy vocabulary.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PromptMode {
    YesNo,
    Single,
    Multi,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionPolicy {
    FirstResponse,
    Unanimous,
    Majority,
    AllRespondents,
}

/// Invalid persisted policy token at a dialogue storage boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialoguePolicyParseError {
    PromptMode,
    ResolutionPolicy,
}

impl std::fmt::Display for DialoguePolicyParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::PromptMode => "Dialogue prompt has an unknown mode",
            Self::ResolutionPolicy => "Dialogue prompt has an unknown resolution policy",
        })
    }
}

impl std::error::Error for DialoguePolicyParseError {}

impl PromptMode {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::YesNo => "YesNo",
            Self::Single => "Single",
            Self::Multi => "Multi",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DialoguePolicyParseError> {
        [Self::YesNo, Self::Single, Self::Multi]
            .into_iter()
            .find(|policy| policy.stable_id() == value)
            .ok_or(DialoguePolicyParseError::PromptMode)
    }
}

impl ResolutionPolicy {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::FirstResponse => "FirstResponse",
            Self::Unanimous => "Unanimous",
            Self::Majority => "Majority",
            Self::AllRespondents => "AllRespondents",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DialoguePolicyParseError> {
        [
            Self::FirstResponse,
            Self::Unanimous,
            Self::Majority,
            Self::AllRespondents,
        ]
        .into_iter()
        .find(|policy| policy.stable_id() == value)
        .ok_or(DialoguePolicyParseError::ResolutionPolicy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_and_persisted_policies_share_one_vocabulary() {
        let mode: PromptMode = serde_json::from_str("\"yes_no\"").unwrap();
        assert_eq!(PromptMode::parse(mode.stable_id()), Ok(mode));
        let resolution: ResolutionPolicy = serde_json::from_str("\"all_respondents\"").unwrap();
        assert_eq!(
            ResolutionPolicy::parse(resolution.stable_id()),
            Ok(resolution)
        );
        assert_eq!(
            PromptMode::parse("FirstResponse"),
            Err(DialoguePolicyParseError::PromptMode)
        );
        assert_eq!(
            ResolutionPolicy::parse("Multi"),
            Err(DialoguePolicyParseError::ResolutionPolicy)
        );
    }
}
