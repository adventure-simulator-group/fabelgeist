// Domain values used at the durable character-name boundary.

use std::fmt;
use adventuresim_core::identity::CharacterId;

/// Failure of the private semantic identity authority, before reducer formatting.
#[derive(Debug)]
pub(crate) enum CharacterNameError {
    MissingCharacter(CharacterId),
    MissingPersonality(CharacterId),
    MissingIdentity(CharacterId),
    InvalidIdentity(serde_json::Error),
    InvalidName(adventuresim_world_schema::person_names::NameCatalogError),
    MissingGeneratedSurname,
    AgePredatesCalendar,
}

impl fmt::Display for CharacterNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCharacter(id) => write!(formatter, "Character {} not found", id),
            Self::MissingPersonality(id) => {
                write!(formatter, "Character {} has no personality", id)
            }
            Self::MissingIdentity(id) => write!(
                formatter,
                "Character {} has no semantic name identity",
                id
            ),
            Self::InvalidIdentity(error) => {
                write!(formatter, "Invalid character name identity: {error}")
            }
            Self::InvalidName(error) => error.fmt(formatter),
            Self::MissingGeneratedSurname => {
                formatter.write_str("Generated German identity has no hereditary surname")
            }
            Self::AgePredatesCalendar => formatter.write_str("Character age predates the calendar"),
        }
    }
}

impl std::error::Error for CharacterNameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidIdentity(error) => Some(error),
            Self::InvalidName(error) => Some(error),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for CharacterNameError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidIdentity(error)
    }
}

impl From<adventuresim_world_schema::person_names::NameCatalogError> for CharacterNameError {
    fn from(error: adventuresim_world_schema::person_names::NameCatalogError) -> Self {
        Self::InvalidName(error)
    }
}

/// Storage adapter for a [`PersonalNameIdentity`] row. Name assignment validates
/// rendering before serialization; consuming code propagates parse failures.
#[derive(Clone, Debug, Eq, PartialEq, spacetimedb::SpacetimeType)]
pub struct NameIdentityJson {
    value: String,
}

impl NameIdentityJson {
    pub(crate) fn from_identity(
        identity: &adventuresim_world_schema::person_names::PersonalNameIdentity,
    ) -> Result<Self, serde_json::Error> {
        serde_json::to_string(identity).map(|value| Self { value })
    }

    pub(crate) fn parse(
        value: &str,
    ) -> Result<adventuresim_world_schema::person_names::PersonalNameIdentity, serde_json::Error>
    {
        serde_json::from_str(value)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for NameIdentityJson {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
