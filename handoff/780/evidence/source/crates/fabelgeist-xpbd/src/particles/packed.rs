//! Native particle records preserve position, mass, and velocity roles.
use crate::ParticleInverseMass;
use anyhow::{Result, ensure};
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_math::Vec3;

/// Native xyz position followed by inverse kilograms, including signed zero.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticlePositionRecord {
    position: [f32; 3],
    pub(super) inverse_mass: ParticleInverseMass,
}

/// Native xyz velocity followed by the unused, zeroed shader word.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleVelocityRecord {
    velocity: [f32; 3],
    padding: f32,
}

/// Positions stay paired with their mass words through upload and readback.
pub struct ParticlePositions(Vec<ParticlePositionRecord>);
impl ParticlePositions {
    pub fn new(positions: &[Vec3], inverse_masses: &[ParticleInverseMass]) -> Result<Self> {
        ensure!(
            positions.len() == inverse_masses.len(),
            "Particles::write: {} positions but {} inverse masses",
            positions.len(),
            inverse_masses.len()
        );
        Ok(Self(
            positions
                .iter()
                .zip(inverse_masses)
                .map(|(p, &inverse_mass)| ParticlePositionRecord {
                    position: [p.x, p.y, p.z],
                    inverse_mass,
                })
                .collect(),
        ))
    }
    pub fn upload(&self) -> BufferUpload<'_> {
        BufferUpload::from_elements(&self.0)
    }
    pub fn positions(&self) -> impl Iterator<Item = Vec3> + '_ {
        self.0
            .iter()
            .map(|p| Vec3::new(p.position[0], p.position[1], p.position[2]))
    }
    pub fn inverse_masses(&self) -> impl Iterator<Item = ParticleInverseMass> + '_ {
        self.0.iter().map(|p| p.inverse_mass)
    }
}
impl From<Vec<ParticlePositionRecord>> for ParticlePositions {
    fn from(records: Vec<ParticlePositionRecord>) -> Self {
        Self(records)
    }
}

/// Velocity records cannot be used as position/mass records.
///
/// ```compile_fail
/// use fabelgeist_xpbd::{ParticlePositions, ParticleVelocities};
/// fn positions(_: ParticlePositions) {}
/// positions(ParticleVelocities::from(&[][..]));
/// ```
pub struct ParticleVelocities(Vec<ParticleVelocityRecord>);
impl ParticleVelocities {
    pub fn at_rest(positions: &ParticlePositions) -> Self {
        Self(vec![ParticleVelocityRecord::default(); positions.0.len()])
    }
    pub fn upload(&self) -> BufferUpload<'_> {
        BufferUpload::from_elements(&self.0)
    }
    pub fn velocities(&self) -> impl Iterator<Item = Vec3> + '_ {
        self.0
            .iter()
            .map(|v| Vec3::new(v.velocity[0], v.velocity[1], v.velocity[2]))
    }
}
impl From<&[Vec3]> for ParticleVelocities {
    fn from(velocities: &[Vec3]) -> Self {
        Self(
            velocities
                .iter()
                .map(|v| ParticleVelocityRecord {
                    velocity: [v.x, v.y, v.z],
                    padding: 0.0,
                })
                .collect(),
        )
    }
}
impl From<Vec<ParticleVelocityRecord>> for ParticleVelocities {
    fn from(records: Vec<ParticleVelocityRecord>) -> Self {
        Self(records)
    }
}
