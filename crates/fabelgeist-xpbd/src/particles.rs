//! Particle state on the GPU.

use anyhow::anyhow;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;

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
    inverse_masses: Vec<f32>,
    count: u32,
    capacity: u32,
}

impl Particles {
    pub fn new(context: &WgpuContext, capacity: u32) -> Result<Self> {
        let capacity = capacity.max(1);
        let bytes = capacity as u64 * 16;
        // `vertex` so that the renderer can draw straight out of the solver's
        // own buffer, with no copy between simulating and showing.
        let definition = BufferDefinition::storage().with_vertex().with_copy_src();
        Ok(Self {
            positions: Buffer::new(
                context,
                bytes,
                definition.clone().with_label("particle positions"),
            )?,
            previous: Buffer::new(
                context,
                bytes,
                definition.clone().with_label("particle previous"),
            )?,
            velocities: Buffer::new(context, bytes, definition.with_label("particle velocities"))?,
            inverse_masses: Vec::new(),
            count: 0,
            capacity,
        })
    }

    /// Allocate and upload in one go.
    pub fn from_positions(
        context: &WgpuContext,
        positions: &[Vec3],
        inverse_masses: &[f32],
    ) -> Result<Self> {
        let mut particles = Self::new(context, positions.len() as u32)?;
        particles.write(context, positions, inverse_masses)?;
        Ok(particles)
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Upload positions and inverse masses, and reset velocities to rest.
    pub fn write(
        &mut self,
        context: &WgpuContext,
        positions: &[Vec3],
        inverse_masses: &[f32],
    ) -> Result<()> {
        if positions.len() != inverse_masses.len() {
            return Err(anyhow!(
                "Particles::write: {} positions but {} inverse masses",
                positions.len(),
                inverse_masses.len()
            ));
        }
        if positions.len() as u32 > self.capacity {
            return Err(anyhow!(
                "Particles::write: capacity is {}, given {}",
                self.capacity,
                positions.len()
            ));
        }

        let packed = pack(positions, inverse_masses);
        self.positions.write(context, &packed)?;
        self.previous.write(context, &packed)?;
        self.velocities
            .write(context, &vec![0.0f32; positions.len() * 4])?;
        self.count = positions.len() as u32;
        self.inverse_masses = inverse_masses.to_vec();
        Ok(())
    }

    /// The inverse masses as last uploaded. Zero pins the particle.
    pub fn inverse_masses(&self) -> &[f32] {
        &self.inverse_masses
    }

    /// Overwrite positions, keeping the inverse masses. Used to re-drape a
    /// garment from a fresh layout without rebuilding the constraints.
    pub fn write_positions(&mut self, context: &WgpuContext, positions: &[Vec3]) -> Result<()> {
        if positions.len() as u32 != self.count {
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
        let raw: Vec<f32> = self.positions.read(context).await?;
        Ok(unpack(&raw, self.count as usize))
    }

    pub async fn read_velocities(&self, context: &WgpuContext) -> Result<Vec<Vec3>> {
        let raw: Vec<f32> = self.velocities.read(context).await?;
        Ok(unpack(&raw, self.count as usize))
    }

    pub async fn read_inverse_masses(&self, context: &WgpuContext) -> Result<Vec<f32>> {
        let raw: Vec<f32> = self.positions.read(context).await?;
        Ok((0..self.count as usize).map(|i| raw[i * 4 + 3]).collect())
    }
}

pub fn pack(positions: &[Vec3], inverse_masses: &[f32]) -> Vec<f32> {
    let mut packed = Vec::with_capacity(positions.len() * 4);
    for (position, &inverse_mass) in positions.iter().zip(inverse_masses) {
        packed.extend_from_slice(&[position.x, position.y, position.z, inverse_mass]);
    }
    packed
}

pub fn unpack(raw: &[f32], count: usize) -> Vec<Vec3> {
    (0..count)
        .map(|i| Vec3::new(raw[i * 4], raw[i * 4 + 1], raw[i * 4 + 2]))
        .collect()
}
