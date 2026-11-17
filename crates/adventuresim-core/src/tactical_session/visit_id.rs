//! A visit distinguishes successive admissions of the same actor and scene.

use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocalSceneVisitId(NonZeroU64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("local scene visit identifiers must be positive")]
pub struct LocalSceneVisitIdentityError;

impl LocalSceneVisitId {
    /// Native database and generated SDK adapters carry the auto-assigned key.
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl TryFrom<u64> for LocalSceneVisitId {
    type Error = LocalSceneVisitIdentityError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        NonZeroU64::new(value)
            .map(Self)
            .ok_or(LocalSceneVisitIdentityError)
    }
}
