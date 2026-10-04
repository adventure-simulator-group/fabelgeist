//! Logical particle cardinality and physical allocation remain distinct.
use super::ParticleError;
use fabelgeist_compute::SortItemCount;
use fabelgeist_gpu::prelude::{BufferByteLength, InvocationCount, PassParameters};

/// Active GPU particle records, retaining their native unsigned word width.
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
impl From<ParticleCount> for usize {
    fn from(particles: ParticleCount) -> Self {
        particles.0 as usize
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
        (self.0 as u64 * std::mem::size_of::<super::ParticlePositionRecord>() as u64).into()
    }
    pub fn invocations(self) -> InvocationCount {
        self.0.into()
    }
    pub fn sort_items(self) -> SortItemCount {
        self.0.into()
    }
    pub fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("count".into(), self.0.into());
    }
    pub(super) fn admit_replacement(self, actual: ParticleInputCount) -> Result<(), ParticleError> {
        if actual.gpu_count() != self {
            return Err(ParticleError::PositionCount {
                expected: self,
                actual,
            });
        }
        Ok(())
    }
}

/// Allocated particle records; an empty request retains one physical sentinel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParticleCapacity(ParticleCount);
impl From<ParticleCount> for ParticleCapacity {
    fn from(particles: ParticleCount) -> Self {
        Self(ParticleCount(particles.0.max(1)))
    }
}
impl From<u32> for ParticleCapacity {
    fn from(particles: u32) -> Self {
        Self::from(ParticleCount::from(particles))
    }
}
impl std::fmt::Display for ParticleCapacity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
impl ParticleCapacity {
    pub fn count(self) -> ParticleCount {
        self.0
    }
    pub fn record_bytes(self) -> BufferByteLength {
        self.0.record_bytes()
    }
    pub fn sort_items(self) -> SortItemCount {
        self.0.sort_items()
    }
    pub(super) fn admit(self, actual: ParticleInputCount) -> Result<(), ParticleError> {
        if actual.gpu_count() > self.0 {
            return Err(ParticleError::Capacity {
                capacity: self,
                actual,
            });
        }
        Ok(())
    }
}

/// Full host input cardinality, before the existing native GPU narrowing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParticleInputCount(usize);
impl From<usize> for ParticleInputCount {
    fn from(particles: usize) -> Self {
        Self(particles)
    }
}

/// Address within an unconstrained host input, before GPU count admission.
/// It retains the full host width even when a malformed input cannot fit u32.
///
/// ```compile_fail
/// use fabelgeist_xpbd::{ParticleIndex, ParticleInputIndex};
/// let _: ParticleIndex = ParticleInputIndex::from(3usize);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParticleInputIndex(usize);
impl From<usize> for ParticleInputIndex {
    fn from(index: usize) -> Self {
        Self(index)
    }
}
impl std::fmt::Display for ParticleInputIndex {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Cardinality of kilogram mass inputs, distinct from inverse-mass inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParticleMassCount(usize);
impl From<usize> for ParticleMassCount {
    fn from(masses: usize) -> Self {
        Self(masses)
    }
}
impl std::fmt::Display for ParticleMassCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
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
    /// Preserve the established native narrowing; this does not prove bounds.
    pub fn gpu_count(self) -> ParticleCount {
        ParticleCount(self.0 as u32)
    }
    pub(super) fn admit_masses(self, masses: InverseMassCount) -> Result<(), ParticleError> {
        if self.0 != masses.0 {
            return Err(ParticleError::InputLength {
                positions: self,
                masses,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InverseMassCount(usize);
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
