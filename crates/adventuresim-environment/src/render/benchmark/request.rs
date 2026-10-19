//! Arrival identity is distinct from a bank or population's content digest.

use std::num::NonZeroU64;

/// A caller-owned serial request ordinal within one renderer lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
#[cfg_attr(feature = "prepared", serde(transparent))]
pub struct ForestRequestId(NonZeroU64);

impl ForestRequestId {
    pub const FIRST: Self = Self(NonZeroU64::MIN);

    /// Sequence/transport boundary; zero is never a submitted request.
    pub fn from_ordinal(ordinal: NonZeroU64) -> Self {
        Self(ordinal)
    }

    pub fn ordinal(self) -> NonZeroU64 {
        self.0
    }

    pub fn successor(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}
