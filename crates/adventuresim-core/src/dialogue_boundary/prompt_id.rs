//! Prompt row identity retains the exact conversation that issued it.

use super::DialogueSessionId;
use serde::{Deserialize, Serialize};

pub(super) const DIALOGUE_PROMPT_SEPARATOR: &str = ":prompt:";

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct DialoguePromptRowId {
    encoded: String,
    session: DialogueSessionId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("invalid dialogue prompt identity")]
pub struct DialoguePromptIdentityError;

impl DialoguePromptRowId {
    pub fn session_id(&self) -> &DialogueSessionId {
        &self.session
    }

    pub fn as_str(&self) -> &str {
        &self.encoded
    }
}

impl TryFrom<String> for DialoguePromptRowId {
    type Error = DialoguePromptIdentityError;

    fn try_from(encoded: String) -> Result<Self, Self::Error> {
        let (session, prompt) = encoded
            .split_once(DIALOGUE_PROMPT_SEPARATOR)
            .ok_or(DialoguePromptIdentityError)?;
        if prompt.is_empty()
            || prompt.contains(DIALOGUE_PROMPT_SEPARATOR)
            || encoded.chars().any(char::is_control)
        {
            return Err(DialoguePromptIdentityError);
        }
        let session = DialogueSessionId::try_from(session.to_owned())
            .map_err(|_| DialoguePromptIdentityError)?;
        Ok(Self { encoded, session })
    }
}

impl From<DialoguePromptRowId> for String {
    fn from(identity: DialoguePromptRowId) -> Self {
        identity.encoded
    }
}
