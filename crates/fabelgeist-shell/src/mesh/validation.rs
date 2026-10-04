//! Validate each admission contract in its established failure order.
use super::{ShellConstraintFamily, ShellMesh, ShellMeshError, ShellParticleReference};
use crate::{BendRecordValidity, MassValidity, ParticleInputCount};
use fabelgeist_math::Vec3;

pub(super) fn validate_positions(positions: &[Vec3]) -> Result<(), ShellMeshError> {
    for (particle, &position) in positions.iter().enumerate() {
        if !position.is_finite() {
            return Err(ShellMeshError::NonFinitePosition {
                particle: particle.into(),
                position,
            });
        }
    }
    Ok(())
}

impl ShellMesh {
    pub fn validate(&self) -> Result<(), ShellMeshError> {
        let particles = ParticleInputCount::from(self.positions.len());
        self.validate_particles(particles)?;
        self.validate_constraint_lengths()?;
        self.validate_particle_references(particles)?;
        self.validate_rest_data()
    }

    fn validate_particles(&self, particles: ParticleInputCount) -> Result<(), ShellMeshError> {
        if self.positions.is_empty() || self.positions.len() > u32::MAX as usize {
            return Err(ShellMeshError::ParticleCount { actual: particles });
        }
        validate_positions(&self.positions)?;
        if self.masses.len() != self.positions.len() {
            return Err(ShellMeshError::MassCount {
                particles,
                masses: self.masses.len().into(),
            });
        }
        for (particle, &mass) in self.masses.iter().enumerate() {
            if mass.validity() != MassValidity::FiniteNonnegative {
                return Err(ShellMeshError::InvalidMass {
                    particle: particle.into(),
                    mass,
                });
            }
        }
        Ok(())
    }

    fn validate_constraint_lengths(&self) -> Result<(), ShellMeshError> {
        for (family, constraints, records) in [
            (
                ShellConstraintFamily::Stretch,
                self.edges.len(),
                self.rest_lengths.len(),
            ),
            (
                ShellConstraintFamily::Bending,
                self.bends.len(),
                self.bend_weights.len(),
            ),
        ] {
            if constraints != records {
                return Err(ShellMeshError::ConstraintDataLength {
                    family,
                    constraints: constraints.into(),
                    records: records.into(),
                });
            }
        }
        Ok(())
    }

    fn validate_particle_references(
        &self,
        particles: ParticleInputCount,
    ) -> Result<(), ShellMeshError> {
        for (family, indices) in [
            (
                ShellParticleReference::Triangle,
                self.triangles.as_flattened(),
            ),
            (ShellParticleReference::Stretch, self.edges.as_flattened()),
            (ShellParticleReference::Seam, self.seams.as_flattened()),
        ] {
            for &index in indices {
                if (index as usize) >= usize::from(particles) {
                    return Err(ShellMeshError::IndexOutOfBounds {
                        family,
                        index: index.into(),
                        particles,
                    });
                }
            }
        }
        for bend in &self.bends {
            for index in bend.particles() {
                if (index as usize) >= usize::from(particles) {
                    return Err(ShellMeshError::BendIndex {
                        index: index.into(),
                        particles,
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_rest_data(&self) -> Result<(), ShellMeshError> {
        for &length in &self.rest_lengths {
            if !length.is_finite() || length < 0.0 {
                return Err(ShellMeshError::RestData {
                    family: ShellConstraintFamily::Stretch,
                });
            }
        }
        for record in &self.bend_weights {
            if record.validity() != BendRecordValidity::Finite {
                return Err(ShellMeshError::RestData {
                    family: ShellConstraintFamily::Bending,
                });
            }
        }
        Ok(())
    }
}
