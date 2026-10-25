//! Particle state on the GPU.

use crate::ParticleInverseMass;
use anyhow::anyhow;
use fabelgeist_gpu::prelude::BufferUse;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;
pub use count::ParticleCount;

mod count;
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
    positions: Buffer,
    /// xyz = position at the start of the substep.
    previous: Buffer,
    /// xyz = velocity.
    velocities: Buffer,
    /// Kept on the host because the inverse masses share their words with the
    /// positions: re-uploading positions alone would otherwise have to read
    /// the buffer back first.
    inverse_masses: Vec<ParticleInverseMass>,
    count: ParticleCount,
    capacity: ParticleCount,
}

impl Particles {
    pub fn new(context: &WgpuContext, capacity: ParticleCount) -> Result<Self> {
        let capacity = capacity.with_sentinel();
        let bytes = capacity.packed_bytes();
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
            count: ParticleCount::default(),
            capacity,
        })
    }

    /// Allocate and upload in one go.
    pub fn from_positions(
        context: &WgpuContext,
        positions: &[Vec3],
        inverse_masses: &[ParticleInverseMass],
    ) -> Result<Self> {
        let mut particles = Self::new(context, ParticleCount::try_from(positions.len())?)?;
        particles.write(context, positions, inverse_masses)?;
        Ok(particles)
    }

    /// Native position/mass records; the held allocation cannot be replaced.
    pub fn positions(&self) -> &Buffer {
        &self.positions
    }

    /// Native previous-position records for the current substep.
    pub fn previous(&self) -> &Buffer {
        &self.previous
    }

    /// Native velocity records for the held population.
    pub fn velocities(&self) -> &Buffer {
        &self.velocities
    }

    pub fn count(&self) -> ParticleCount {
        self.count
    }

    pub fn capacity(&self) -> ParticleCount {
        self.capacity
    }

    /// Upload positions and inverse masses, and reset velocities to rest.
    pub fn write(
        &mut self,
        context: &WgpuContext,
        positions: &[Vec3],
        inverse_masses: &[ParticleInverseMass],
    ) -> Result<()> {
        if positions.len() != inverse_masses.len() {
            return Err(anyhow!(
                "Particles::write: {} positions but {} inverse masses",
                positions.len(),
                inverse_masses.len()
            ));
        }
        let count = ParticleCount::try_from(positions.len())?;
        if count > self.capacity {
            return Err(anyhow!(
                "Particles::write: capacity is {}, given {}",
                self.capacity,
                positions.len()
            ));
        }

        let packed = ParticlePositions::new(positions, inverse_masses)?;
        self.positions.write(context, packed.upload())?;
        self.previous.write(context, packed.upload())?;
        self.velocities
            .write(context, ParticleVelocities::at_rest(&packed).upload())?;
        self.count = count;
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
        if ParticleCount::try_from(positions.len())? != self.count {
            return Err(anyhow!(
                "Particles::write_positions: holds {} particles, given {}",
                self.count,
                positions.len()
            ));
        }
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
        Ok(records.inverse_masses().take(usize::from(self.count)).collect())
    }
}

#[cfg(test)]
mod tests;
