// Domain values used at the durable character-name boundary.

use std::fmt;

/// A durable character-table identity at the reducer boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct CharacterId(u64);

impl CharacterId {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for CharacterId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<CharacterId> for u64 {
    fn from(value: CharacterId) -> Self {
        value.get()
    }
}

/// Deterministic seed owned by a generated character's name projection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct NameSeed(u64);

impl NameSeed {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for NameSeed {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// Absolute strategic minute used to derive a name's historical period.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct WorldMinute(u64);

impl WorldMinute {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for WorldMinute {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// Validated serialized form of a [`PersonalNameIdentity`] row.
#[derive(Clone, Debug, Eq, PartialEq, spacetimedb::SpacetimeType)]
pub struct NameIdentityJson {
    value: String,
}

impl NameIdentityJson {
    pub(crate) fn from_identity(
        identity: &adventuresim_world_schema::person_names::PersonalNameIdentity,
    ) -> Result<Self, String> {
        serde_json::to_string(identity)
            .map(|value| Self { value })
            .map_err(|error| format!("Could not serialize character name identity: {error}"))
    }

    pub(crate) fn parse(
        value: &str,
    ) -> Result<adventuresim_world_schema::person_names::PersonalNameIdentity, String> {
        serde_json::from_str(value)
            .map_err(|error| format!("Could not parse character name identity: {error}"))
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
