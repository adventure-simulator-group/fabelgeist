//! Counts retain their host or native particle role through admission.
use super::ParticlePositionRecord;
use anyhow::{Result, ensure};
use fabelgeist_gpu::prelude::{BufferByteLength, PassParameter};

/// Particle records in a native GPU prefix, distinct from allocated capacity.
/// This cardinality does not prove that the records belong to an allocation.
///
/// ```compile_fail
/// use fabelgeist_xpbd::{ParticleCapacity, ParticleCount};
/// fn active(_: ParticleCount) {}
/// active(ParticleCapacity::from(4));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParticleCount(u32);
impl From<u32> for ParticleCount {
    fn from(particles: u32) -> Self {
        Self(particles)
    }
}
impl From<ParticleCount> for u32 {
    fn from(particles: ParticleCount) -> Self {
        particles.0
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
impl std::fmt::Display for ParticleCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
impl ParticleCount {
    pub const EMPTY: Self = Self(0);
    pub fn record_bytes(self) -> BufferByteLength {
        (u64::from(self.0) * std::mem::size_of::<ParticlePositionRecord>() as u64).into()
    }
    pub(super) fn admit_replacement(self, actual: ParticleInputCount) -> Result<()> {
        ensure!(
            actual.gpu_count() == self,
            "Particles::write_positions: holds {} particles, given {}",
            self,
            actual
        );
        Ok(())
    }
}

/// Physical particle allocation; an empty request reserves one sentinel record.
/// Capacity never establishes the number of currently uploaded particles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParticleCapacity(u32);
impl From<u32> for ParticleCapacity {
    fn from(particles: u32) -> Self {
        Self(particles.max(1))
    }
}
impl From<ParticleCount> for ParticleCapacity {
    fn from(particles: ParticleCount) -> Self {
        Self::from(particles.0)
    }
}
impl From<ParticleCapacity> for u32 {
    fn from(capacity: ParticleCapacity) -> Self {
        capacity.0
    }
}
impl From<ParticleCapacity> for usize {
    fn from(capacity: ParticleCapacity) -> Self {
        capacity.0 as usize
    }
}
impl std::fmt::Display for ParticleCapacity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
impl ParticleCapacity {
    pub fn contains(self, particles: ParticleCount) -> bool {
        particles.0 <= self.0
    }
    pub fn record_bytes(self) -> BufferByteLength {
        (u64::from(self.0) * std::mem::size_of::<ParticlePositionRecord>() as u64).into()
    }
    /// One native unsigned index word per allocated particle.
    pub fn index_bytes(self) -> BufferByteLength {
        (u64::from(self.0) * std::mem::size_of::<u32>() as u64).into()
    }
    pub(super) fn admit(self, actual: ParticleInputCount) -> Result<()> {
        ensure!(
            self.contains(actual.gpu_count()),
            "Particles::write: capacity is {}, given {}",
            self,
            actual
        );
        Ok(())
    }
}

/// Full host cardinality, before the native GPU's unsigned-word narrowing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParticleInputCount(usize);
impl From<usize> for ParticleInputCount {
    fn from(particles: usize) -> Self {
        Self(particles)
    }
}
impl From<ParticleInputCount> for usize {
    fn from(particles: ParticleInputCount) -> Self {
        particles.0
    }
}
impl std::fmt::Display for ParticleInputCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
impl ParticleInputCount {
    /// Truncate to the native word width, without proving allocation or bounds.
    pub fn gpu_count(self) -> ParticleCount {
        ParticleCount(self.0 as u32)
    }
    pub(super) fn admit_masses(self, masses: InverseMassCount) -> Result<()> {
        ensure!(
            self.0 == masses.0,
            "Particles::write: {} positions but {} inverse masses",
            self,
            masses
        );
        Ok(())
    }
}

/// The parallel mass input has its own admission and diagnostic role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct InverseMassCount(usize);
impl From<usize> for InverseMassCount {
    fn from(masses: usize) -> Self {
        Self(masses)
    }
}
impl std::fmt::Display for InverseMassCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests;
