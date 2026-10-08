//! Particle state on the GPU.

use crate::ParticleInverseMass;
mod quantities;
use fabelgeist_gpu::prelude::BufferUse;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;
use quantities::InverseMassCount;
pub use quantities::{ParticleCapacity, ParticleCount, ParticleInputCount};
mod packed;
pub use packed::{
    ParticlePositionRecord, ParticlePositions, ParticleVelocities, ParticleVelocityRecord,
};

/// Positions, the previous substep's positions, and velocities.
///
/// Position and inverse mass share a buffer -- `xyz` and `w` -- because a
/// constraint solve reads both and nothing else, and one load beats two. An
/// inverse mass of zero pins the particle: every correction is scaled by it,
/// so a pinned particle never moves and the solver needs no special case.
/// Since the two are interleaved, changing what is pinned means going through
/// [`Particles::write`] with both.
pub struct Particles {
    /// xyz = position, w = inverse mass. Also usable as a vertex buffer.
    pub positions: Buffer,
    /// xyz = position at the start of the substep.
    pub previous: Buffer,
    /// xyz = velocity.
    pub velocities: Buffer,
    /// Kept on the host because the inverse masses share their words with the
    /// positions: re-uploading positions alone would otherwise have to read
    /// the buffer back first.
    inverse_masses: Vec<ParticleInverseMass>,
    count: ParticleCount,
    capacity: ParticleCapacity,
}

impl Particles {
    pub fn new(context: &WgpuContext, capacity: ParticleCapacity) -> Result<Self> {
        let bytes = capacity.record_bytes();
        // `vertex` so that the renderer can draw straight out of the solver's
        // own buffer, with no copy between simulating and showing.
        let definition = BufferDefinition::storage()
            .with_usage(BufferUse::Vertex)
            .with_usage(BufferUse::CopySource);
        Ok(Self {
            positions: Buffer::new(
                context,
                bytes,
                definition.clone().with_label(("particle positions").into()),
            )?,
            previous: Buffer::new(
                context,
                bytes,
                definition.clone().with_label(("particle previous").into()),
            )?,
            velocities: Buffer::new(
                context,
                bytes,
                definition.with_label(("particle velocities").into()),
            )?,
            inverse_masses: Vec::new(),
            count: ParticleCount::EMPTY,
            capacity,
        })
    }

    /// Allocate and upload in one go.
    pub fn from_positions(
        context: &WgpuContext,
        positions: &[Vec3],
        inverse_masses: &[ParticleInverseMass],
    ) -> Result<Self> {
        let mut particles = Self::new(
            context,
            ParticleInputCount::from(positions.len()).gpu_count().into(),
        )?;
        particles.write(context, positions, inverse_masses)?;
        Ok(particles)
    }

    pub fn count(&self) -> ParticleCount {
        self.count
    }

    pub fn capacity(&self) -> ParticleCapacity {
        self.capacity
    }

    /// Upload positions and inverse masses, and reset velocities to rest.
    pub fn write(
        &mut self,
        context: &WgpuContext,
        positions: &[Vec3],
        inverse_masses: &[ParticleInverseMass],
    ) -> Result<()> {
        let count = ParticleInputCount::from(positions.len());
        count.admit_masses(InverseMassCount::from(inverse_masses.len()))?;
        self.capacity.admit(count)?;

        let packed = ParticlePositions::new(positions, inverse_masses)?;
        self.positions.write(context, packed.upload())?;
        self.previous.write(context, packed.upload())?;
        self.velocities
            .write(context, ParticleVelocities::at_rest(&packed).upload())?;
        self.count = count.gpu_count();
        self.inverse_masses = inverse_masses.to_vec();
        Ok(())
    }

    /// The inverse masses as last uploaded. Zero pins the particle.
    pub fn inverse_masses(&self) -> &[ParticleInverseMass] {
        &self.inverse_masses
    }

    /// Overwrite positions, keeping the inverse masses. Used to re-drape a
    /// garment from a fresh layout without rebuilding the constraints.
    pub fn write_positions(&mut self, context: &WgpuContext, positions: &[Vec3]) -> Result<()> {
        self.count
            .admit_replacement(ParticleInputCount::from(positions.len()))?;
        let masses = std::mem::take(&mut self.inverse_masses);
        let result = self.write(context, positions, &masses);
        self.inverse_masses = masses;
        result
    }

    pub async fn read_positions(&self, context: &WgpuContext) -> Result<Vec<Vec3>> {
        let records = ParticlePositions::from(
            self.positions
                .read::<ParticlePositionRecord>(context)
                .await?,
        );
        Ok(records.positions().take(usize::from(self.count)).collect())
    }

    pub async fn read_velocities(&self, context: &WgpuContext) -> Result<Vec<Vec3>> {
        let records = ParticleVelocities::from(
            self.velocities
                .read::<ParticleVelocityRecord>(context)
                .await?,
        );
        Ok(records.velocities().take(usize::from(self.count)).collect())
    }

    pub async fn read_inverse_masses(
        &self,
        context: &WgpuContext,
    ) -> Result<Vec<ParticleInverseMass>> {
        let records = ParticlePositions::from(
            self.positions
                .read::<ParticlePositionRecord>(context)
                .await?,
        );
        Ok(records
            .inverse_masses()
            .take(usize::from(self.count))
            .collect())
    }
}

#[cfg(test)]
mod tests;
