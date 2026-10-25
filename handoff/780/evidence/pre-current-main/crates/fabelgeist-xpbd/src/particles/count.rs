//! Live particle population, distinct from constraint incidence and addresses.

use fabelgeist_compute::SortItemCount;
use fabelgeist_gpu::prelude::{BufferByteLength, InvocationCount, PassParameter};
use std::num::TryFromIntError;

/// Particles held or allocated by the transient solver, including an empty set.
#[derive(Clone, Copy, Debug, Default, derive_more::Display, derive_more::From,
    PartialEq, Eq, PartialOrd, Ord)]
pub struct ParticleCount(u32);

impl TryFrom<usize> for ParticleCount {
    type Error = TryFromIntError;

    fn try_from(particles: usize) -> Result<Self, Self::Error> {
        u32::try_from(particles).map(Self)
    }
}

impl ParticleCount {
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(crate) fn with_sentinel(self) -> Self {
        Self(self.0.max(1))
    }

    /// Byte extent of native position/mass or velocity records.
    pub fn packed_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.0) * 16)
    }

    /// Fixed-width native representation for owned grid-layout arithmetic.
    pub const fn native_elements(self) -> u32 {
        self.0
    }
}

impl From<ParticleCount> for usize {
    fn from(particles: ParticleCount) -> Self {
        particles.0 as usize
    }
}

impl From<ParticleCount> for PassParameter {
    fn from(particles: ParticleCount) -> Self {
        Self::Unsigned(particles.0)
    }
}

impl From<ParticleCount> for InvocationCount {
    fn from(particles: ParticleCount) -> Self {
        Self::from(particles.0)
    }
}

impl From<ParticleCount> for SortItemCount {
    fn from(particles: ParticleCount) -> Self {
        Self::from(particles.0)
    }
}
