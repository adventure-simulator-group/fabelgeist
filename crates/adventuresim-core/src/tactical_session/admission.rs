//! Enrollment generation distinguishes rejoining the same serving connection.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub struct TacticalAdmissionGeneration {
    value: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("tactical admission generation is exhausted")]
pub struct TacticalAdmissionGenerationError;

impl TacticalAdmissionGeneration {
    pub const fn initial() -> Self {
        Self { value: 0 }
    }

    pub const fn is_initial(self) -> bool {
        self.value == 0
    }

    pub fn successor(self) -> Result<Self, TacticalAdmissionGenerationError> {
        self.value
            .checked_add(1)
            .map(|value| Self { value })
            .ok_or(TacticalAdmissionGenerationError)
    }
}
