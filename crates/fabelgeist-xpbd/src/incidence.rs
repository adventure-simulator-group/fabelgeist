//! Particle incidence acquires its solver role before coloring or upload.
use fabelgeist_gpu::prelude::{
    Buffer, BufferCreationError, BufferDefinition, BufferUpload, WgpuContext,
};
use std::num::NonZeroUsize;

/// Address of a solver particle, independent of draw or constraint addresses.
/// Admission assigns a role, not membership in any particular particle buffer.
///
/// ```compile_fail
/// use fabelgeist_xpbd::{ConstraintIndex, ParticleIndex};
/// let _: ConstraintIndex = ParticleIndex::from(3);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ParticleIndex(u32);
impl From<u32> for ParticleIndex {
    fn from(word: u32) -> Self {
        Self(word)
    }
}
impl From<ParticleIndex> for usize {
    fn from(index: ParticleIndex) -> Self {
        index.0 as usize
    }
}
impl std::fmt::Display for ParticleIndex {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Number of particle entries in a flattened constraint input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstraintParticleCount(usize);
impl std::fmt::Display for ConstraintParticleCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A fixed constraint always touches at least one particle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstraintArity(NonZeroUsize);
impl TryFrom<usize> for ConstraintArity {
    type Error = ConstraintLayoutError;
    fn try_from(arity: usize) -> Result<Self, ConstraintLayoutError> {
        NonZeroUsize::new(arity)
            .map(Self)
            .ok_or(ConstraintLayoutError::ZeroArity)
    }
}
impl std::fmt::Display for ConstraintArity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstraintLayoutError {
    ZeroArity,
    Incomplete {
        particles: ConstraintParticleCount,
        arity: ConstraintArity,
    },
}
impl std::fmt::Display for ConstraintLayoutError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroArity => formatter.write_str("ConstraintSet: arity must be at least one"),
            Self::Incomplete { particles, arity } => write!(
                formatter,
                "ConstraintSet: {particles} indices is not a whole number of arity-{arity} constraints"
            ),
        }
    }
}
impl std::error::Error for ConstraintLayoutError {}

/// Complete, immutable fixed-arity constraints in their producer's order.
#[derive(Clone, Debug)]
pub struct ConstraintIncidence {
    particles: Vec<ParticleIndex>,
    arity: ConstraintArity,
}
impl ConstraintIncidence {
    /// Decode native particle-address records, preserving their exact order.
    pub fn from_native_records<const N: usize>(
        records: &[[u32; N]],
    ) -> Result<Self, ConstraintLayoutError> {
        Self::from_native_flat(records.as_flattened(), ConstraintArity::try_from(N)?)
    }

    /// Decode a flat native representation only when every record is complete.
    pub fn from_native_flat(
        words: &[u32],
        arity: ConstraintArity,
    ) -> Result<Self, ConstraintLayoutError> {
        if !words.len().is_multiple_of(arity.0.get()) {
            return Err(ConstraintLayoutError::Incomplete {
                particles: ConstraintParticleCount(words.len()),
                arity,
            });
        }
        Ok(Self {
            particles: words.iter().copied().map(ParticleIndex::from).collect(),
            arity,
        })
    }
    pub fn arity(&self) -> ConstraintArity {
        self.arity
    }
    pub(crate) fn records(&self) -> impl Iterator<Item = &[ParticleIndex]> {
        self.particles.chunks(self.arity.0.get())
    }
    pub(crate) fn upload_ordered(
        &self,
        context: &WgpuContext,
        coloring: &crate::Coloring,
    ) -> Result<Buffer, BufferCreationError> {
        let mut words = Vec::with_capacity(self.particles.len());
        for &constraint in coloring.order() {
            let first = usize::from(constraint) * self.arity.0.get();
            for particle in &self.particles[first..first + self.arity.0.get()] {
                // This is the constraint particle buffer's native u32 ABI.
                words.push(particle.0);
            }
        }
        Buffer::from_upload(
            context,
            BufferUpload::from_elements(&words).with_empty_word(),
            BufferDefinition::storage().with_label("constraint particles".into()),
        )
    }
}

/// The two-particle records used by distance and spring constraints.
#[derive(Clone, Debug, Default)]
pub struct ConstraintEdges(Vec<[ParticleIndex; 2]>);
impl From<&[[u32; 2]]> for ConstraintEdges {
    fn from(records: &[[u32; 2]]) -> Self {
        let mut edges = Vec::with_capacity(records.len());
        for &record in records {
            edges.push(record.map(ParticleIndex::from));
        }
        Self(edges)
    }
}
impl ConstraintEdges {
    pub fn pairs(&self) -> &[[ParticleIndex; 2]] {
        &self.0
    }
    pub fn count(&self) -> crate::ConstraintCount {
        self.0.len().into()
    }
    pub fn incidence(&self) -> ConstraintIncidence {
        let mut particles = Vec::with_capacity(self.0.len() * 2);
        for edge in &self.0 {
            particles.extend_from_slice(edge);
        }
        ConstraintIncidence {
            particles,
            arity: ConstraintArity(NonZeroUsize::new(2).unwrap()),
        }
    }
}

#[cfg(test)]
mod tests;
