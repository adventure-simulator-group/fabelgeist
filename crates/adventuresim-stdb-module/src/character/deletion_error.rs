//! Character cleanup keeps persistent-character refusal and custody failures.

use crate::inventory_container::InventoryContainerError;
use adventuresim_core::identity::CharacterId;

#[derive(Debug)]
pub(crate) enum CharacterDeletionError {
    PersistentCharacter(CharacterId),
    UnexpectedPartyMember,
    Container(Box<InventoryContainerError>),
}

impl From<InventoryContainerError> for CharacterDeletionError {
    fn from(source: InventoryContainerError) -> Self {
        Self::Container(Box::new(source))
    }
}

impl std::fmt::Display for CharacterDeletionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PersistentCharacter(_) => {
                f.write_str("Refusing to cascade-delete a persistent character")
            }
            Self::UnexpectedPartyMember => {
                f.write_str("Temporary character party contains another member")
            }
            Self::Container(source) => source.fmt(f),
        }
    }
}

impl std::error::Error for CharacterDeletionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Container(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}
