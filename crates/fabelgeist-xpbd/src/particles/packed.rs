//! Complete position/mass and velocity records retain their native GPU roles.
use super::{InverseMassCount, ParticleCount, ParticleError, ParticleInputCount};
use crate::ParticleInverseMass;
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_math::Vec3;

/// Native xyz position followed by inverse mass; all words retain their bits.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticlePositionRecord {
    position: [f32; 3],
    inverse_mass: ParticleInverseMass,
}
impl ParticlePositionRecord {
    pub(super) fn inverse_mass(&self) -> ParticleInverseMass {
        self.inverse_mass
    }
    fn position(&self) -> Vec3 {
        Vec3::new(self.position[0], self.position[1], self.position[2])
    }
}

/// Native xyz velocity followed by an unused word; uploads clear that word.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleVelocityRecord {
    velocity: [f32; 3],
    padding: f32,
}
impl From<Vec3> for ParticleVelocityRecord {
    fn from(velocity: Vec3) -> Self {
        Self {
            velocity: [velocity.x, velocity.y, velocity.z],
            padding: 0.0,
        }
    }
}
impl ParticleVelocityRecord {
    fn velocity(&self) -> Vec3 {
        Vec3::new(self.velocity[0], self.velocity[1], self.velocity[2])
    }
}

/// Position records stay paired with their mass words through upload/readback.
///
/// ```compile_fail
/// use fabelgeist_xpbd::{ParticlePositions, ParticleVelocities};
/// fn positions(_: ParticlePositions) {}
/// positions(ParticleVelocities::at_rest(0usize.into()));
/// ```
pub struct ParticlePositions(Vec<ParticlePositionRecord>);
impl ParticlePositions {
    /// Construct complete native records; a parallel mass mismatch is rejected.
    pub fn new(
        positions: &[Vec3],
        inverse_masses: &[ParticleInverseMass],
    ) -> Result<Self, ParticleError> {
        ParticleInputCount::from(positions.len())
            .admit_masses(InverseMassCount::from(inverse_masses.len()))?;
        let mut records = Vec::with_capacity(positions.len());
        for (position, &inverse_mass) in positions.iter().zip(inverse_masses) {
            records.push(ParticlePositionRecord {
                position: [position.x, position.y, position.z],
                inverse_mass,
            });
        }
        Ok(Self(records))
    }
    pub fn upload(&self) -> BufferUpload<'_> {
        BufferUpload::from_elements(&self.0)
    }
    pub fn positions(&self, count: ParticleCount) -> Vec<Vec3> {
        self.0[..usize::from(count)]
            .iter()
            .map(ParticlePositionRecord::position)
            .collect()
    }
}
impl From<Vec<ParticlePositionRecord>> for ParticlePositions {
    fn from(records: Vec<ParticlePositionRecord>) -> Self {
        Self(records)
    }
}

pub struct ParticleVelocities(Vec<ParticleVelocityRecord>);
impl ParticleVelocities {
    pub fn at_rest(count: ParticleInputCount) -> Self {
        Self(vec![ParticleVelocityRecord::default(); usize::from(count)])
    }
    pub fn upload(&self) -> BufferUpload<'_> {
        BufferUpload::from_elements(&self.0)
    }
    pub fn velocities(&self, count: ParticleCount) -> Vec<Vec3> {
        self.0[..usize::from(count)]
            .iter()
            .map(ParticleVelocityRecord::velocity)
            .collect()
    }
}
impl From<&[Vec3]> for ParticleVelocities {
    fn from(velocities: &[Vec3]) -> Self {
        Self(
            velocities
                .iter()
                .copied()
                .map(ParticleVelocityRecord::from)
                .collect(),
        )
    }
}
impl From<Vec<ParticleVelocityRecord>> for ParticleVelocities {
    fn from(records: Vec<ParticleVelocityRecord>) -> Self {
        Self(records)
    }
}
