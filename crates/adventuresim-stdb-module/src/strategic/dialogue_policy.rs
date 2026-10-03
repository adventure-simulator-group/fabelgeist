//! Typed decoding of persisted prompt policies at the storage boundary.
use super::DialoguePrompt;
use adventuresim_dialogue::{DialoguePolicyParseError, PromptMode, ResolutionPolicy};

impl DialoguePrompt {
    pub(super) fn policies(
        &self,
    ) -> Result<(PromptMode, ResolutionPolicy), DialoguePolicyParseError> {
        Ok((
            PromptMode::parse(&self.mode)?,
            ResolutionPolicy::parse(&self.resolution_policy)?,
        ))
    }
}
