//! A garment on the GPU: particles, the constraints holding it together, and
//! the loop that steps it.

use anyhow::anyhow;
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;
use fabelgeist_physics::Collisions;
use fabelgeist_xpbd::{ConstraintSet, Particles, Solver, SolverSettings, SubstepHook};

use crate::fabric::Fabric;
use crate::garment::GarmentMesh;
use crate::selfcollision::SelfCollision;
use crate::wgsl;

/// A simulable garment.
pub struct Cloth {
    pub particles: Particles,
    pub stretch: ConstraintSet,
    pub bending: ConstraintSet,
    pub seams: ConstraintSet,
    pub self_collision: SelfCollision,
    pub fabric: Fabric,

    /// The triangle list, unchanged by simulation. What the renderer draws,
    /// indexing straight into `particles.positions`.
    pub triangles: Vec<[u32; 3]>,
    /// Kept so a garment can be reset to its flat layout.
    initial_positions: Vec<Vec3>,
    inverse_masses: Vec<f32>,
}

impl Cloth {
    pub fn new(
        context: &WgpuContext,
        cache: &KernelCache,
        mesh: &GarmentMesh,
        fabric: Fabric,
    ) -> Result<Self> {
        if mesh.positions.is_empty() {
            return Err(anyhow!("Cloth: the garment has no particles"));
        }

        let inverse_masses = mesh.inverse_masses();
        let particles = Particles::from_positions(context, &mesh.positions, &inverse_masses)?;

        let stretch = ConstraintSet::distance(
            context,
            cache,
            "stretch",
            &mesh.edges,
            &mesh.rest_lengths,
            fabric.stretch_compliance,
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
            fabric.seam_compliance,
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
            fabric.bend_compliance,
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
            mesh.positions.len() as u32,
            &mesh.adjacency(),
            fabric.particle_radius(),
        )?;

        Ok(Self {
            particles,
            stretch,
            bending,
            seams,
            self_collision,
            fabric,
            triangles: mesh.triangles.clone(),
            initial_positions: mesh.positions.clone(),
            inverse_masses,
        })
    }

    pub fn particle_count(&self) -> u32 {
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

    /// Push the fabric's compliances into the constraint sets. Call after
    /// changing [`Cloth::fabric`]; the masses are baked in at build time and
    /// need a rebuild.
    pub fn apply_fabric(&mut self) {
        self.stretch.compliance = self.fabric.stretch_compliance;
        self.bending.compliance = self.fabric.bend_compliance;
        self.seams.compliance = self.fabric.seam_compliance;
        self.self_collision
            .set_radius(self.fabric.particle_radius());
    }

    /// Solver settings that suit this fabric.
    pub fn settings(&self) -> SolverSettings {
        SolverSettings {
            damping: self.fabric.damping,
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
        collisions.particle_radius = self.fabric.particle_radius();

        let mut hook = ClothHook {
            collisions,
            self_collision: &mut self.self_collision,
            substep: 0,
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
    pub fn step_interleaved(
        &mut self,
        context: &WgpuContext,
        solver: &Solver,
        collisions: &mut Collisions,
        delta: f32,
    ) -> Result<()> {
        collisions.particle_radius = self.fabric.particle_radius();

        let mut hook = ClothHook {
            collisions,
            self_collision: &mut self.self_collision,
            substep: 0,
        };

        solver.step_interleaved(
            context,
            &self.particles,
            &mut [&mut self.stretch, &mut self.seams, &mut self.bending],
            &mut hook,
            delta,
        )
    }

    pub async fn read_positions(&self, context: &WgpuContext) -> Result<Vec<Vec3>> {
        self.particles.read_positions(context).await
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
struct ClothHook<'a> {
    collisions: &'a mut Collisions,
    self_collision: &'a mut SelfCollision,
    /// Which substep of the current step this is. The spatial hash is rebuilt
    /// on the first one only -- see [`SelfCollision::record`].
    substep: u32,
}

impl SubstepHook for ClothHook<'_> {
    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: f32,
    ) -> Result<()> {
        let _ = substep;
        Collisions::record(self.collisions, batch, particles)?;
        let rebuild = self.substep == 0;
        self.substep += 1;
        self.self_collision.record(batch, particles, rebuild)
    }
}

#[cfg(test)]
mod tests;
