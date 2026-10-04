//! Host projection bridges complete GPU state and the CPU contact solver.
use super::*;

impl SurfaceContacts {
    /// Project a completed GPU substep before the next integration begins.
    /// Particle spheres alone cannot detect triangle-interior crossings.
    /// Resolve relative normal velocity with mass-weighted impulses; uploading
    /// through `write_positions` would incorrectly reset every velocity.
    /// `interval_start` holds positions from before several GPU substeps;
    /// without it only the last substep's motion is swept.
    pub async fn project_particles(
        &self,
        context: &fabelgeist_gpu::globals::WgpuContext,
        particles: &fabelgeist_xpbd::Particles,
        thickness: f32,
        iterations: u32,
        interval_start: Option<&[Vec3]>,
    ) -> Result<SurfaceContactCount, SurfaceProjectionError> {
        let mut previous = match interval_start {
            Some(start) => {
                let actual = fabelgeist_xpbd::ParticleInputCount::from(start.len());
                if usize::from(actual) != usize::from(particles.count()) {
                    return Err(SurfaceProjectionError::IntervalCount {
                        expected: particles.count(),
                        actual,
                    });
                }
                start.to_vec()
            }
            None => {
                let raw: Vec<fabelgeist_xpbd::ParticlePositionRecord> =
                    particles.previous.read(context).await.map_err(
                        |source: fabelgeist_gpu::prelude::ReadbackError| -> SurfaceProjectionError {
                            SurfaceProjectionError::PreviousReadback {
                                source: Box::new(source),
                            }
                        },
                    )?;
                fabelgeist_xpbd::ParticlePositions::from(raw).positions(particles.count())
            }
        };
        let predicted = particles.read_positions(context).await.map_err(
            |source: fabelgeist_xpbd::ParticleError| -> SurfaceProjectionError {
                SurfaceProjectionError::ParticleState {
                    stage: SurfaceProjectionStage::PositionsRead,
                    source,
                }
            },
        )?;
        let mut corrected = predicted.clone();
        let mut velocities = particles.read_velocities(context).await.map_err(
            |source: fabelgeist_xpbd::ParticleError| -> SurfaceProjectionError {
                SurfaceProjectionError::ParticleState {
                    stage: SurfaceProjectionStage::VelocitiesRead,
                    source,
                }
            },
        )?;
        let count = corrected.len();
        let mut masses = particles.inverse_masses().to_vec();
        previous.extend_from_slice(&self.static_positions);
        corrected.extend_from_slice(&self.static_positions);
        masses.resize(corrected.len(), ParticleInverseMass::PINNED);
        velocities.resize(corrected.len(), Vec3::default());
        let contacts = self.solve_inner(
            &mut corrected,
            &previous,
            &masses,
            thickness,
            iterations,
            Some(&mut velocities),
        );
        if contacts > SurfaceContactCount::NONE {
            particles.positions.write(
                context,
                fabelgeist_xpbd::ParticlePositions::new(
                    &corrected[..count],
                    particles.inverse_masses(),
                )
                .map_err(
                    |source: fabelgeist_xpbd::ParticleError| -> SurfaceProjectionError {
                        SurfaceProjectionError::ParticleState {
                            stage: SurfaceProjectionStage::PositionEncoding,
                            source,
                        }
                    },
                )?
                .upload(),
            );
            particles.velocities.write(
                context,
                fabelgeist_xpbd::ParticleVelocities::from(&velocities[..count]).upload(),
            );
        }
        Ok(contacts)
    }
}

#[cfg(test)]
mod tests;
