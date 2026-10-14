//! A thin surface on the GPU: particles, the constraints holding it together, and
//! the loop that steps it.

use anyhow::anyhow;
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;
use fabelgeist_physics::Collisions;
use fabelgeist_xpbd::{ConstraintSet, Particles, Solver, SolverSettings, SubstepHook};

use crate::ShellMaterial;
use crate::ShellMesh;
use crate::selfcollision::SelfCollision;
use crate::wgsl;

/// When the host resolves swept contacts and outer layers between GPU
/// substeps. Each host projection reads particles back and waits for the GPU,
/// so this schedule dominates the cost of an interleaved step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostContactSchedule {
    /// GPU substeps swept by one host projection. Zero leaves contact to the
    /// GPU collider and self-collision kernels.
    pub interval_substeps: u32,
    /// Projection sweeps in one surface contact solve.
    pub iterations: u32,
    /// Alternations between an outer layer and surface contacts.
    pub outer_layer_passes: u32,
}

impl Default for HostContactSchedule {
    fn default() -> Self {
        Self {
            interval_substeps: 1,
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
    pub fn new(
        context: &WgpuContext,
        cache: &KernelCache,
        mesh: &ShellMesh,
        material: ShellMaterial,
    ) -> Result<Self> {
        mesh.validate()?;
        material.validate()?;
        if mesh.positions.is_empty() {
            return Err(anyhow!("Shell: the garment has no particles"));
        }

        let inverse_masses = mesh.inverse_masses();
        let particles = Particles::from_positions(context, &mesh.positions, &inverse_masses)?;

        let stretch = ConstraintSet::distance(
            context,
            cache,
            "stretch",
            &mesh.edges,
            &mesh.rest_lengths,
            material.stretch_compliance,
        )?;

        // Seams start at zero rest length -- the two edges are meant to become
        // one -- and the solver closes the gap over the first few frames. That
        // pulling-together is the whole drape: it is what turns flat panels
        // into a garment shaped round a body.
        let seam_rest = vec![0.0f32; mesh.seams.len()];
        let seams = ConstraintSet::distance(
            context,
            cache,
            "seams",
            &mesh.seams,
            &seam_rest,
            material.seam_compliance,
        )?;

        let bend_particles: Vec<u32> = mesh
            .bends
            .iter()
            .flat_map(|bend| bend.particles())
            .collect();
        let bend_kernel = cache.get(
            context,
            &fabelgeist_xpbd::wgsl::constraint_kernel(wgsl::BEND),
        )?;
        let mut bending = ConstraintSet::new(
            context,
            "bending",
            bend_kernel,
            cache,
            &bend_particles,
            4,
            material.bend_compliance,
        )?;
        // The weights are per constraint and the set is stored in colour
        // order, so they have to be permuted the same way -- otherwise every
        // hinge reads another hinge's weights.
        let ordered_weights = bending.reorder(&mesh.bend_weights);
        let flat_weights: Vec<f32> = ordered_weights.into_iter().flatten().collect();
        bending.attach_raw(context, "weights", &flat_weights)?;

        let self_collision = SelfCollision::new(
            context,
            cache,
            particles.count(),
            &mesh.adjacency(),
            material.particle_radius(),
        )?;

        Ok(Self {
            outer_layer: None,
            host_contacts: HostContactSchedule::default(),
            surface_contacts: crate::surface_contact::SurfaceContacts::new(
                mesh.positions.len().into(),
                mesh.triangles.clone(),
            )
            .with_seams(&mesh.seams),
            particles,
            stretch,
            bending,
            seams,
            self_collision,
            material,
            triangles: mesh.triangles.clone(),
            initial_positions: mesh.positions.clone(),
            inverse_masses,
        })
    }

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
    pub fn reset(&mut self, context: &WgpuContext) -> Result<()> {
        self.particles
            .write(context, &self.initial_positions, &self.inverse_masses)
    }

    /// Push the material's compliances into the constraint sets. Call after
    /// changing [`Shell::material`]; the masses are baked in at build time and
    /// need a rebuild.
    pub fn apply_material(&mut self) {
        self.stretch.compliance = self.material.stretch_compliance;
        self.bending.compliance = self.material.bend_compliance;
        self.seams.compliance = self.material.seam_compliance;
        self.self_collision
            .set_radius(self.material.particle_radius());
    }

    /// Solver settings that suit this material.
    pub fn settings(&self) -> SolverSettings {
        SolverSettings {
            damping: self.material.damping,
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
        delta: f32,
    ) -> Result<()> {
        collisions.particle_radius = self.material.particle_radius();

        let mut hook = ShellHook {
            collisions,
            self_collision: &mut self.self_collision,
        };

        solver.record_step(
            batch,
            &self.particles,
            &mut [&mut self.stretch, &mut self.seams, &mut self.bending],
            &mut hook,
            delta,
        )
    }

    pub fn step(
        &mut self,
        context: &WgpuContext,
        solver: &Solver,
        collisions: &mut Collisions,
        delta: f32,
    ) -> Result<()> {
        let mut batch = KernelBatch::labelled(context, "cloth step");
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
        delta: f32,
    ) -> Result<()> {
        if !delta.is_finite() || delta <= 0.0 {
            return Ok(());
        }
        let count = solver.settings.substeps.max(1);
        let substep = delta / count as f32;
        let schedule = self.host_contacts;
        let interval = schedule.interval_substeps;
        collisions.particle_radius = self.material.particle_radius();
        let mut interval_start = None;
        for index in 0..count {
            if interval > 1 && index % interval == 0 {
                // Sweep the whole interval, not only its last GPU substep.
                interval_start = Some(self.particles.read_positions(context).await?);
            }
            let mut batch = KernelBatch::labelled(context, "surface contact substep");
            let mut hook = ShellHook {
                collisions,
                self_collision: &mut self.self_collision,
            };
            solver.record_substep(
                &mut batch,
                &self.particles,
                &mut [&mut self.stretch, &mut self.seams, &mut self.bending],
                &mut hook,
                substep,
            )?;
            batch.submit();
            if interval > 0 && ((index + 1) % interval == 0 || index + 1 == count) {
                self.project_host_contacts(context, schedule, interval_start.take().as_deref())
                    .await?;
            }
        }
        if interval == 0 {
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
    ) -> Result<()> {
        // Alternate the outer layer and swept self contacts. Neither may
        // silently win merely because it is the last positional correction.
        for _ in 0..schedule.outer_layer_passes.max(1) {
            if let Some(layer) = &self.outer_layer {
                layer
                    .project_particles(context, &self.particles, &self.triangles)
                    .await?;
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
                    .await?;
            } else {
                context.submitted_work_done().await;
            }
            let Some(layer) = &self.outer_layer else {
                break;
            };
            let positions = self.particles.read_positions(context).await?;
            if layer.surface_residual(&positions, &self.triangles)
                <= crate::outer_layer::CLEARANCE_TOLERANCE
            {
                break;
            }
        }
        Ok(())
    }
    pub async fn read_positions(&self, context: &WgpuContext) -> Result<Vec<Vec3>> {
        self.particles.read_positions(context).await
    }

    /// Snapshot the deformed surface as reusable mesh geometry. This reads back
    /// particle state; it is intended for export/CSG rather than a per-frame draw.
    pub async fn read_mesh(&self, context: &WgpuContext) -> Result<fabelgeist_mesh::MeshData> {
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
    pub async fn is_finite(&self, context: &WgpuContext) -> Result<bool> {
        Ok(self
            .read_positions(context)
            .await?
            .iter()
            .all(|p| p.is_finite()))
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
    fn after_solve(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        _substep: f32,
    ) -> Result<()> {
        // Sewing and bending can pull a particle through the body after the
        // prediction collision pass. Resolve that before storing velocity.
        self.self_collision.record(batch, particles, true)?;
        Collisions::record(self.collisions, batch, particles)
    }

    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: f32,
    ) -> Result<()> {
        let _ = substep;
        Collisions::record(self.collisions, batch, particles)?;
        self.self_collision.record(batch, particles, true)
    }
}
