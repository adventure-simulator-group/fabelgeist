//! Particle incidence acquires its solver role before coloring or upload.
use derive_more::{Display, Error, From};
use fabelgeist_gpu::prelude::{Buffer, BufferDefinition, BufferUpload, WgpuContext};
use std::num::NonZeroUsize;

/// Address of a solver particle, independent of draw or constraint addresses.
/// Admission assigns a role, not membership in any particular particle buffer.
///
/// ```compile_fail
/// use fabelgeist_xpbd::{ConstraintIndex, ParticleIndex};
/// let _: ConstraintIndex = ParticleIndex::from(3);
/// ```
#[derive(Clone, Copy, Debug, Display, From, PartialEq, Eq, Hash)]
pub struct ParticleIndex(u32);

/// Number of particle entries in a flattened constraint input.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub struct ConstraintParticleCount(usize);

/// A fixed constraint always touches at least one particle.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub struct ConstraintArity(NonZeroUsize);

/// A checked view of complete, immutable fixed-arity particle records.
#[derive(Clone, Debug)]
pub struct ConstraintIncidence<'a> {
    particles: &'a [ParticleIndex],
    arity: ConstraintArity,
}

/// The two-particle records used by distance and spring constraints.
#[derive(Clone, Debug, Default)]
pub struct ConstraintEdges(Vec<[ParticleIndex; 2]>);

#[derive(Clone, Debug, Display, Error, PartialEq, Eq)]
pub enum ConstraintLayoutError {
    #[display("ConstraintSet: arity must be at least one")]
    ZeroArity,
    #[display(
        "ConstraintSet: {particles} indices is not a whole number of arity-{arity} constraints"
    )]
    Incomplete {
        particles: ConstraintParticleCount,
        arity: ConstraintArity,
    },
}

impl From<ParticleIndex> for usize {
    fn from(index: ParticleIndex) -> Self {
        index.0 as usize
    }
}

impl ConstraintArity {
    // A native edge record contains two particle addresses.
    const PAIR: Self = Self(NonZeroUsize::MIN.saturating_add(1));
}

impl TryFrom<usize> for ConstraintArity {
    type Error = ConstraintLayoutError;
    fn try_from(arity: usize) -> Result<Self, ConstraintLayoutError> {
        NonZeroUsize::new(arity)
            .map(Self)
            .ok_or(ConstraintLayoutError::ZeroArity)
    }
}

impl<'a> ConstraintIncidence<'a> {
    /// Borrow particle-address records, preserving their exact order.
    pub fn from_records<const N: usize>(
        records: &'a [[ParticleIndex; N]],
    ) -> Result<Self, ConstraintLayoutError> {
        Self::new(records.as_flattened(), ConstraintArity::try_from(N)?)
    }

    /// Borrow flat particle storage only when every record is complete.
    pub fn new(
        particles: &'a [ParticleIndex],
        arity: ConstraintArity,
    ) -> Result<Self, ConstraintLayoutError> {
        if !particles.len().is_multiple_of(arity.0.get()) {
            return Err(ConstraintLayoutError::Incomplete {
                particles: ConstraintParticleCount(particles.len()),
                arity,
            });
        }
        Ok(Self { particles, arity })
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
    ) -> anyhow::Result<Buffer> {
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
    pub fn incidence(&self) -> ConstraintIncidence<'_> {
        ConstraintIncidence {
            particles: self.0.as_flattened(),
            arity: ConstraintArity::PAIR,
        }
    }
}

#[cfg(test)]
mod tests;
