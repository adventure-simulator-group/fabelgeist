//! A thin surface on the GPU: particles, the constraints holding it together, and
//! the loop that steps it.

use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;
use fabelgeist_physics::Collisions;
use fabelgeist_xpbd::{
    ConstraintSet, Particles, Solver, SolverSettings, SolverStepError, SolverSubstepError,
    StepActivity, StepDuration, SubstepCadence, SubstepCount, SubstepCycleBoundary,
    SubstepDuration, SubstepHook,
};

use crate::ShellMaterial;
use crate::selfcollision::SelfCollision;
mod build;
mod error;
pub use error::{
    ShellBuildError, ShellHookError, ShellPositionValidity, ShellProjectionError,
    ShellProjectionStage, ShellStepError,
};

/// When the host resolves swept contacts and outer layers between GPU
/// substeps. Each host projection reads particles back and waits for the GPU,
/// so this schedule dominates the cost of an interleaved step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostContactSchedule {
    /// GPU substeps swept by one host projection. Zero leaves contact to the
    /// GPU collider and self-collision kernels.
    pub interval_substeps: SubstepCount,
    /// Projection sweeps in one surface contact solve.
    pub iterations: u32,
    /// Alternations between an outer layer and surface contacts.
    pub outer_layer_passes: u32,
}

impl Default for HostContactSchedule {
    fn default() -> Self {
        Self {
            interval_substeps: SubstepCount::ONE,
            iterations: 4,
            outer_layer_passes: 4,
        }
    }
}

/// A simulable thin surface, shared by cloth and metal.
pub struct Shell {
    pub outer_layer: Option<crate::outer_layer::OuterLayer>,
    pub host_contacts: HostContactSchedule,
    pub particles: Particles,
    surface_contacts: crate::surface_contact::SurfaceContacts,
    pub stretch: ConstraintSet,
    pub bending: ConstraintSet,
    /// Zero-length attachments; cloth uses these for seams.
    pub seams: ConstraintSet,
    pub self_collision: SelfCollision,
    pub material: ShellMaterial,

    /// The triangle list, unchanged by simulation. Particle positions are
    /// interleaved with inverse masses; use `read_mesh` for packed mesh data.
    pub triangles: Vec<[u32; 3]>,
    /// Kept so the surface can be reset to its initial layout.
    initial_positions: Vec<Vec3>,
    inverse_masses: Vec<crate::ParticleInverseMass>,
}

impl Shell {
    pub fn set_collision_surface(
        &mut self,
        positions: &[Vec3],
        faces: &[[u32; 3]],
        clearance: f32,
    ) {
        self.surface_contacts
            .set_static_surface(positions, faces, clearance);
    }

    pub fn particle_count(&self) -> fabelgeist_xpbd::ParticleCount {
        self.particles.count()
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Put every particle back where the flat layout placed it, at rest.
    pub fn reset(
        &mut self,
        context: &WgpuContext,
    ) -> std::result::Result<(), fabelgeist_xpbd::ParticleError> {
        self.particles
            .write(context, &self.initial_positions, &self.inverse_masses)
    }

    /// Push the material's compliances into the constraint sets. Call after
    /// changing [`Shell::material`]; the masses are baked in at build time and
    /// need a rebuild.
    pub fn apply_material(&mut self) {
        self.stretch.compliance = self.material.stretch_compliance.into();
        self.bending.compliance = self.material.bend_compliance.into();
        self.seams.compliance = self.material.seam_compliance.into();
        self.self_collision
            .set_radius(self.material.particle_radius());
    }

    /// Solver settings that suit this material.
    pub fn settings(&self) -> SolverSettings {
        SolverSettings {
            damping: self.material.damping.into(),
            ..Default::default()
        }
    }

    /// Step the garment: collisions against the world, cloth against itself,
    /// then the material constraints.
    pub fn record_step(
        &mut self,
        batch: &mut KernelBatch,
        solver: &Solver,
        collisions: &mut Collisions,
        delta: StepDuration,
    ) -> std::result::Result<(), ShellStepError> {
        collisions.particle_radius = self.material.particle_radius();

        let mut hook = ShellHook {
            collisions,
            self_collision: &mut self.self_collision,
        };

        solver
            .record_step(
                batch,
                &self.particles,
                &mut [&mut self.stretch, &mut self.seams, &mut self.bending],
                &mut hook,
                delta,
            )
            .map_err(ShellStepError::Solver)
    }

    pub fn step(
        &mut self,
        context: &WgpuContext,
        solver: &Solver,
        collisions: &mut Collisions,
        delta: StepDuration,
    ) -> std::result::Result<(), ShellStepError> {
        let mut batch = KernelBatch::labelled(context, ("cloth step").into());
        self.record_step(&mut batch, solver, collisions, delta)?;
        batch.submit();
        Ok(())
    }

    /// Step with one submission per substep -- see
    /// [`Solver::step_interleaved`].
    ///
    /// This is the one to call when the solver shares a device with whatever
    /// is drawing the result, which for an interactive fit it always does.
    pub async fn step_interleaved(
        &mut self,
        context: &WgpuContext,
        solver: &Solver,
        collisions: &mut Collisions,
        delta: StepDuration,
    ) -> std::result::Result<(), ShellStepError> {
        if delta.finite_activity() == StepActivity::Inactive {
            return Ok(());
        }
        let count = solver.settings.substeps.at_least_one();
        let substep = delta.for_substeps(count);
        let schedule = self.host_contacts;
        let interval = schedule.interval_substeps;
        collisions.particle_radius = self.material.particle_radius();
        let mut interval_start = None;
        for index in count.sequence() {
            if interval.cadence() == SubstepCadence::Multiple
                && index.starts_cycle(interval) == SubstepCycleBoundary::Boundary
            {
                // Sweep the whole interval, not only its last GPU substep.
                interval_start = Some(
                    self.particles
                        .read_positions(context)
                        .await
                        .map_err(ShellStepError::IntervalRead)?,
                );
            }
            let mut batch = KernelBatch::labelled(context, ("surface contact substep").into());
            let mut hook = ShellHook {
                collisions,
                self_collision: &mut self.self_collision,
            };
            solver
                .record_substep(
                    &mut batch,
                    &self.particles,
                    &mut [&mut self.stretch, &mut self.seams, &mut self.bending],
                    &mut hook,
                    substep,
                )
                .map_err(
                    |source: SolverSubstepError<ShellHookError>| -> ShellStepError {
                        ShellStepError::Solver(SolverStepError {
                            substep: index,
                            source,
                        })
                    },
                )?;
            batch.submit();
            if index.ends_cycle(interval, count) == SubstepCycleBoundary::Boundary {
                self.project_host_contacts(context, schedule, interval_start.take().as_deref())
                    .await
                    .map_err(ShellStepError::Projection)?;
            }
        }
        if interval.cadence() == SubstepCadence::Disabled {
            // Bound the queue: a caller stepping on a timer must not outrun the GPU.
            context.submitted_work_done().await;
        }
        Ok(())
    }

    async fn project_host_contacts(
        &self,
        context: &WgpuContext,
        schedule: HostContactSchedule,
        interval_start: Option<&[Vec3]>,
    ) -> std::result::Result<(), ShellProjectionError> {
        // Alternate the outer layer and swept self contacts. Neither may
        // silently win merely because it is the last positional correction.
        for _ in 0..schedule.outer_layer_passes.max(1) {
            if let Some(layer) = &self.outer_layer {
                layer
                    .project_particles(context, &self.particles, &self.triangles)
                    .await
                    .map_err(
                        |source: fabelgeist_xpbd::ParticleError| -> ShellProjectionError {
                            ShellProjectionError::ParticleState {
                                stage: ShellProjectionStage::OuterLayer,
                                source,
                            }
                        },
                    )?;
            }
            if self.self_collision.enabled {
                self.surface_contacts
                    .project_particles(
                        context,
                        &self.particles,
                        self.material.thickness,
                        schedule.iterations,
                        interval_start,
                    )
                    .await
                    .map_err(ShellProjectionError::SurfaceContact)?;
            } else {
                context.submitted_work_done().await;
            }
            let Some(layer) = &self.outer_layer else {
                break;
            };
            let positions = self.particles.read_positions(context).await.map_err(
                |source: fabelgeist_xpbd::ParticleError| -> ShellProjectionError {
                    ShellProjectionError::ParticleState {
                        stage: ShellProjectionStage::ResidualPositions,
                        source,
                    }
                },
            )?;
            if layer.surface_residual(&positions, &self.triangles)
                <= crate::outer_layer::CLEARANCE_TOLERANCE
            {
                break;
            }
        }
        Ok(())
    }
    pub async fn read_positions(
        &self,
        context: &WgpuContext,
    ) -> std::result::Result<Vec<Vec3>, fabelgeist_xpbd::ParticleError> {
        self.particles.read_positions(context).await
    }

    /// Snapshot the deformed surface as reusable mesh geometry. This reads back
    /// particle state; it is intended for export/CSG rather than a per-frame draw.
    pub async fn read_mesh(
        &self,
        context: &WgpuContext,
    ) -> std::result::Result<fabelgeist_mesh::MeshData, fabelgeist_xpbd::ParticleError> {
        let positions = self.read_positions(context).await?;
        let mut normals = vec![Vec3::default(); positions.len()];
        for &[a, b, c] in &self.triangles {
            let n = (positions[b as usize] - positions[a as usize])
                .cross(positions[c as usize] - positions[a as usize]);
            for i in [a, b, c] {
                normals[i as usize] += n;
            }
        }
        Ok(fabelgeist_mesh::MeshData {
            positions: positions.iter().map(|p| [p.x, p.y, p.z]).collect(),
            normals: normals
                .into_iter()
                .map(|n| {
                    let n = if n.length_squared() > 1e-20 {
                        n.normalize()
                    } else {
                        Vec3::new(0.0, 1.0, 0.0)
                    };
                    [n.x, n.y, n.z]
                })
                .collect(),
            indices: Some(self.triangles.iter().flatten().copied().collect()),
            ..Default::default()
        })
    }

    /// Whether the solve has gone unstable. A garment that has blown up shows
    /// up as a non-finite position, and it is worth catching before it reaches
    /// a renderer.
    pub async fn position_validity(
        &self,
        context: &WgpuContext,
    ) -> std::result::Result<ShellPositionValidity, fabelgeist_xpbd::ParticleError> {
        for position in self.read_positions(context).await? {
            if !position.is_finite() {
                return Ok(ShellPositionValidity::NonFinite);
            }
        }
        Ok(ShellPositionValidity::Finite)
    }
}

/// Body collision then self-collision, in that order.
///
/// Body first: the body is not going to move out of the way, so resolving the
/// cloth against it first means the self-collision pass is working with
/// positions that are already outside the body rather than pushing layers into
/// it.
struct ShellHook<'a> {
    collisions: &'a mut Collisions,
    self_collision: &'a mut SelfCollision,
}

impl SubstepHook for ShellHook<'_> {
    type Error = ShellHookError;
    fn after_solve(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        _substep: SubstepDuration,
    ) -> std::result::Result<(), Self::Error> {
        // Sewing and bending can pull a particle through the body after the
        // prediction collision pass. Resolve that before storing velocity.
        self.self_collision
            .record(batch, particles, true)
            .map_err(ShellHookError::self_collision)?;
        Collisions::record(self.collisions, batch, particles).map_err(ShellHookError::body)
    }

    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: SubstepDuration,
    ) -> std::result::Result<(), Self::Error> {
        let _ = substep;
        Collisions::record(self.collisions, batch, particles).map_err(ShellHookError::body)?;
        self.self_collision
            .record(batch, particles, true)
            .map_err(ShellHookError::self_collision)
    }
}

#[cfg(test)]
mod tests;
