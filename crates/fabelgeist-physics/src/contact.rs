//! The substep hook: everything a particle must stay out of.
//!
//! This is what plugs into [`fabelgeist_xpbd::Solver`]. It runs after the
//! prediction and before the material constraints, and it resolves purely by
//! moving positions -- the substep's own velocity update turns those moves
//! into velocity changes, so there is no impulse arithmetic here at all.

use fabelgeist_gpu::prelude::BufferUpload;
use std::sync::Arc;

use anyhow::anyhow;
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_xpbd::{Particles, SubstepHook};

use crate::collider::{Collider, pack_colliders};
use crate::mesh::MeshCollider;
use crate::wgsl;
use count::{ColliderCapacity, ColliderCapacityFit, ColliderCount};

mod count;

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
    pub fn new(context: &WgpuContext, cache: &KernelCache) -> Result<Self> {
        Ok(Self {
            analytic_kernel: cache.get(context, &wgsl::analytic_source())?,
            mesh_kernel: cache.get(context, &MeshCollider::kernel_source())?,
            colliders: Vec::new(),
            collider_buffer: Buffer::new(
                context,
                ColliderCapacity::INITIAL.byte_length(),
                BufferDefinition::storage().with_label(("colliders").into()),
            )?,
            collider_capacity: ColliderCapacity::INITIAL,
            mesh: None,
            particle_radius: 0.0,
            enabled: true,
        })
    }

    pub fn colliders(&self) -> &[Collider] {
        &self.colliders
    }

    fn collider_count(&self) -> ColliderCount {
        ColliderCount::from(self.colliders.len())
    }

    /// Replace the analytic collider list and upload it.
    pub fn set_colliders(&mut self, context: &WgpuContext, colliders: Vec<Collider>) -> Result<()> {
        let count = ColliderCount::from(colliders.len());
        if self.collider_capacity.fit(count) == ColliderCapacityFit::GrowthRequired {
            self.collider_capacity = ColliderCapacity::for_count(count);
            self.collider_buffer = Buffer::new(
                context,
                self.collider_capacity.byte_length(),
                BufferDefinition::storage().with_label(("colliders").into()),
            )?;
        }
        if !colliders.is_empty() {
            self.collider_buffer.write(
                context,
                BufferUpload::from_elements(&pack_colliders(&colliders)),
            )?;
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
    ) -> Result<()> {
        let held = self.collider_count();
        let provided = ColliderCount::from(colliders.len());
        if provided != held {
            return Err(anyhow!(
                "Collisions::update_colliders: holds {} colliders, given {}; use `set_colliders` to change the count",
                held,
                provided
            ));
        }
        if !colliders.is_empty() {
            self.collider_buffer.write(
                context,
                BufferUpload::from_elements(&pack_colliders(colliders)),
            )?;
        }
        self.colliders = colliders.to_vec();
        Ok(())
    }

    pub fn set_mesh(&mut self, mesh: Option<MeshCollider>) {
        self.mesh = mesh;
    }

    /// Record the resolve passes for one substep.
    pub fn record(&self, batch: &mut KernelBatch, particles: &Particles) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let count = particles.count();
        if count == fabelgeist_xpbd::ParticleCount::EMPTY {
            return Ok(());
        }

        if self.collider_count() != ColliderCount::EMPTY {
            let mut parameters = PassParameters::new();
            parameters.insert("positions", particles.positions.clone());
            parameters.insert("previous", particles.previous.clone());
            parameters.insert("colliders", self.collider_buffer.clone());
            parameters.insert("count", count);
            self.collider_count().bind(&mut parameters);
            parameters.insert("particle_radius", self.particle_radius);
            parameters.insert("pad", 0u32);
            batch.dispatch_items(&self.analytic_kernel, &parameters, u32::from(count))?;
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
    ) -> Result<()> {
        let Some(mesh) = &self.mesh else {
            return Ok(());
        };
        let mut parameters = PassParameters::new();
        parameters.insert("positions", particles.positions.clone());
        parameters.insert("previous", particles.previous.clone());
        mesh.bind(&mut parameters);
        parameters.insert("count", particles.count());
        parameters.insert("thickness", mesh.surface.thickness + self.particle_radius);
        parameters.insert("friction", mesh.surface.friction);
        parameters.insert("search_radius", search_radius);
        batch.dispatch_items(&self.mesh_kernel, &parameters, u32::from(particles.count()))?;
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
    ) -> Result<()> {
        if particles.count() == fabelgeist_xpbd::ParticleCount::EMPTY {
            return Ok(());
        }
        // The pass reads `previous` for friction and for the swept box. Making
        // it equal to the current position means no motion to rub against and
        // a box centred on the particle, which is what a static recovery
        // wants.
        batch.copy_buffer(
            &particles.positions,
            &particles.previous,
            u64::from(particles.count().record_bytes()),
        )?;
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
    ) -> Result<()> {
        let mut batch = KernelBatch::labelled(context, "push out");
        for _ in 0..passes {
            self.record_push_out(&mut batch, particles, search_radius)?;
        }
        batch.submit();
        Ok(())
    }
}

impl SubstepHook for Collisions {
    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        _substep: f32,
    ) -> Result<()> {
        Collisions::record(self, batch, particles)
    }
}

/// Run several hooks in order, so that collisions and, say, a self-collision
/// pass can both sit in the same substep.
pub struct HookChain<'a> {
    pub hooks: Vec<&'a mut dyn SubstepHook>,
}

impl<'a> HookChain<'a> {
    pub fn new(hooks: Vec<&'a mut dyn SubstepHook>) -> Self {
        Self { hooks }
    }
}

impl SubstepHook for HookChain<'_> {
    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: f32,
    ) -> Result<()> {
        for hook in self.hooks.iter_mut() {
            hook.record(batch, particles, substep)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
