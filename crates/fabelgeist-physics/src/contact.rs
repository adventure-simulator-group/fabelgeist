//! The substep hook: everything a particle must stay out of.
//!
//! This is what plugs into [`fabelgeist_xpbd::Solver`]. It runs after the
//! prediction and before the material constraints, and it resolves purely by
//! moving positions -- the substep's own velocity update turns those moves
//! into velocity changes, so there is no impulse arithmetic here at all.

use std::sync::Arc;

use fabelgeist_compute::kernel::{Kernel, KernelBatch, KernelCache};
use fabelgeist_gpu::prelude::{
    Buffer, BufferCreationError, BufferUpload, PassParameters, WgpuContext,
};
use fabelgeist_xpbd::{Particles, SubstepDuration, SubstepHook};

use crate::collider::{Collider, pack_colliders};
use crate::mesh::MeshCollider;

mod count;
mod error;
mod kernel;
use count::ColliderCapacityFit;
pub use count::{ColliderCapacity, ColliderCount};
pub use error::{ColliderUpdateError, CollisionBuildError, CollisionRecordError};
pub use kernel::CollisionKernel;

/// Analytic shapes plus at most one triangle mesh.
///
/// One mesh rather than a list because that is what the case needs -- a body
/// is one mesh -- and because a second would want a broad phase over meshes
/// that nothing yet asks for. Analytic colliders are unlimited and cheap.
pub struct Collisions {
    analytic_kernel: Arc<Kernel>,
    mesh_kernel: Arc<Kernel>,

    colliders: Vec<Collider>,
    collider_buffer: Buffer,
    collider_capacity: ColliderCapacity,

    pub mesh: Option<MeshCollider>,

    /// Half the fabric's thickness: how far a particle's centre is held off
    /// every analytic surface, on top of that surface's own thickness.
    pub particle_radius: f32,
    /// Skip collision entirely without tearing anything down.
    pub enabled: bool,
}

impl Collisions {
    pub fn new(context: &WgpuContext, cache: &KernelCache) -> Result<Self, CollisionBuildError> {
        let capacity = ColliderCapacity::INITIAL;
        Ok(Self {
            analytic_kernel: CollisionKernel::Analytic.load(context, cache)?,
            mesh_kernel: CollisionKernel::Mesh.load(context, cache)?,
            colliders: Vec::new(),
            collider_buffer: capacity.allocate(context).map_err(
                |source: BufferCreationError| -> CollisionBuildError {
                    CollisionBuildError::Allocation { capacity, source }
                },
            )?,
            collider_capacity: capacity,
            mesh: None,
            particle_radius: 0.0,
            enabled: true,
        })
    }

    pub fn colliders(&self) -> &[Collider] {
        &self.colliders
    }

    /// Replace the analytic collider list and upload it.
    pub fn set_colliders(
        &mut self,
        context: &WgpuContext,
        colliders: Vec<Collider>,
    ) -> Result<(), ColliderUpdateError> {
        let count = ColliderCount::from(colliders.len());
        if self.collider_capacity.fit(count) == ColliderCapacityFit::GrowthRequired {
            self.collider_capacity = ColliderCapacity::for_count(count);
            self.collider_buffer = self.collider_capacity.allocate(context).map_err(
                |source: BufferCreationError| -> ColliderUpdateError {
                    ColliderUpdateError::Allocation {
                        capacity: self.collider_capacity,
                        source,
                    }
                },
            )?;
        }
        if !colliders.is_empty() {
            self.collider_buffer.write(
                context,
                BufferUpload::from_elements(&pack_colliders(&colliders)),
            );
        }
        self.colliders = colliders;
        Ok(())
    }

    /// Move the shapes without changing how many there are -- an animated
    /// skeleton driving a set of capsules, frame by frame.
    pub fn update_colliders(
        &mut self,
        context: &WgpuContext,
        colliders: &[Collider],
    ) -> Result<(), ColliderUpdateError> {
        let held = ColliderCount::from(self.colliders.len());
        let provided = ColliderCount::from(colliders.len());
        if held != provided {
            return Err(ColliderUpdateError::Count { held, provided });
        }
        if !colliders.is_empty() {
            self.collider_buffer.write(
                context,
                BufferUpload::from_elements(&pack_colliders(colliders)),
            );
        }
        self.colliders = colliders.to_vec();
        Ok(())
    }

    pub fn set_mesh(&mut self, mesh: Option<MeshCollider>) {
        self.mesh = mesh;
    }

    /// Record the resolve passes for one substep.
    pub fn record(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
    ) -> Result<(), CollisionRecordError> {
        if !self.enabled {
            return Ok(());
        }
        let count = particles.count();
        if count == fabelgeist_xpbd::ParticleCount::EMPTY {
            return Ok(());
        }

        if !self.colliders.is_empty() {
            let mut parameters = PassParameters::new();
            parameters.insert("positions".into(), (particles.positions.clone()).into());
            parameters.insert("previous".into(), (particles.previous.clone()).into());
            parameters.insert("colliders".into(), (self.collider_buffer.clone()).into());
            count.bind(&mut parameters);
            ColliderCount::from(self.colliders.len()).bind(&mut parameters);
            parameters.insert("particle_radius".into(), (self.particle_radius).into());
            parameters.insert("pad".into(), (0u32).into());
            CollisionKernel::Analytic.dispatch(
                batch,
                &self.analytic_kernel,
                &parameters,
                count.invocations(),
            )?;
        }

        if self.mesh.is_some() {
            self.record_mesh(batch, particles, 0.0)?;
        }

        Ok(())
    }

    fn record_mesh(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        search_radius: f32,
    ) -> Result<(), CollisionRecordError> {
        let Some(mesh) = &self.mesh else {
            return Ok(());
        };
        let mut parameters = PassParameters::new();
        parameters.insert("positions".into(), (particles.positions.clone()).into());
        parameters.insert("previous".into(), (particles.previous.clone()).into());
        mesh.bind(&mut parameters);
        particles.count().bind(&mut parameters);
        parameters.insert(
            "thickness".into(),
            (mesh.surface.thickness + self.particle_radius).into(),
        );
        parameters.insert("friction".into(), (mesh.surface.friction).into());
        parameters.insert("search_radius".into(), (search_radius).into());
        CollisionKernel::Mesh.dispatch(
            batch,
            &self.mesh_kernel,
            &parameters,
            particles.count().invocations(),
        )?;
        Ok(())
    }

    /// Lift particles that start *inside* the mesh back out to its surface.
    ///
    /// A substep only looks as far as the collision shell and the distance the
    /// particle just travelled, which is what keeps it cheap. A particle sunk
    /// deep inside the body finds no triangle in that range at all and is left
    /// where it is -- and a garment's initial layout puts plenty of panels
    /// through the body before the first step ever runs.
    ///
    /// So this is a separate pass with a search radius wide enough to reach
    /// the surface from inside. Run it after laying a garment out, and repeat
    /// it a few times: each pass moves a particle to the nearest surface it
    /// can see, which for a deeply buried one may not be the final answer.
    pub fn record_push_out(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        search_radius: f32,
    ) -> Result<(), CollisionRecordError> {
        if particles.count() == fabelgeist_xpbd::ParticleCount::EMPTY {
            return Ok(());
        }
        // The pass reads `previous` for friction and for the swept box. Making
        // it equal to the current position means no motion to rub against and
        // a box centred on the particle, which is what a static recovery
        // wants.
        batch
            .copy_buffer(
                &particles.positions,
                &particles.previous,
                particles.count().record_bytes(),
            )
            .map_err(CollisionRecordError::PreviousPositionsCopy)?;
        self.record_mesh(batch, particles, search_radius)?;
        Ok(())
    }

    /// [`Collisions::record_push_out`], on its own batch, repeated.
    pub fn push_out(
        &self,
        context: &WgpuContext,
        particles: &Particles,
        search_radius: f32,
        passes: u32,
    ) -> Result<(), CollisionRecordError> {
        let mut batch = KernelBatch::labelled(context, ("push out").into());
        for _ in 0..passes {
            self.record_push_out(&mut batch, particles, search_radius)?;
        }
        batch.submit();
        Ok(())
    }
}

impl SubstepHook for Collisions {
    type Error = CollisionRecordError;
    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        _substep: SubstepDuration,
    ) -> Result<(), Self::Error> {
        Collisions::record(self, batch, particles)
    }
}

#[cfg(test)]
mod tests;
