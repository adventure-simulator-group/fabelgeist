//! Dialogue identity grammar shared by trusted scene and HTTP adapters.

use super::prompt_id::DIALOGUE_PROMPT_SEPARATOR;
use serde::{Deserialize, Serialize};

const MAX_DIALOGUE_SESSION_ID_BYTES: usize = 160;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct DialogueSessionId {
    encoded: String,
    owner_character_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("invalid dialogue session identity")]
pub struct DialogueSessionIdentityError;

impl DialogueSessionId {
    pub fn as_str(&self) -> &str {
        &self.encoded
    }

    /// Character storage/transport adapters use the existing scalar key.
    pub const fn owner_character_id(&self) -> u64 {
        self.owner_character_id
    }
}

impl TryFrom<String> for DialogueSessionId {
    type Error = DialogueSessionIdentityError;

    fn try_from(encoded: String) -> Result<Self, Self::Error> {
        if encoded.len() > MAX_DIALOGUE_SESSION_ID_BYTES
            || encoded.chars().any(char::is_control)
            || encoded.contains(DIALOGUE_PROMPT_SEPARATOR)
        {
            return Err(DialogueSessionIdentityError);
        }
        let mut parts = encoded.splitn(3, ':');
        let domain = parts.next();
        let owner = parts.next().and_then(|part| part.parse::<u64>().ok());
        let nonce = parts.next().filter(|part| !part.is_empty());
        match (domain, owner, nonce) {
            (Some("dialogue"), Some(owner_character_id), Some(_)) => Ok(Self {
                encoded,
                owner_character_id,
            }),
            _ => Err(DialogueSessionIdentityError),
        }
    }
}

impl From<DialogueSessionId> for String {
    fn from(identity: DialogueSessionId) -> Self {
        identity.encoded
    }
}
