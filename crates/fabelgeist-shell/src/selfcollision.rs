//! Cloth against itself, through a uniform spatial hash.
//!
//! Rebuilt from scratch every substep: hash each particle into a bucket, sort
//! by bucket, record where each bucket starts, then have every particle look
//! at the 27 buckets around it. A hierarchy would be the wrong tool -- the
//! particles are all the same size and roughly evenly spread, which is the one
//! case a grid wins outright.

use std::sync::Arc;

use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_xpbd::{
    ParticleCapacity, ParticleCount, ParticleInputCount, Particles, SubstepDuration, SubstepHook,
};
mod build;
mod error;
mod hash_layout;
mod resource;
pub use error::{SelfCollisionBuildError, SelfCollisionRecordError};
use hash_layout::CollisionTableSize;
pub use resource::{SelfCollisionBuffer, SelfCollisionKernel};

/// The hash grid, its buffers, and the passes over them.
pub struct SelfCollision {
    hash: Arc<Kernel>,
    clear_ranges: Arc<Kernel>,
    cell_ranges: Arc<Kernel>,
    collide: Arc<Kernel>,
    apply: Arc<Kernel>,
    sort: RadixSort,

    cells: Buffer,
    indices: Buffer,
    starts: Buffer,
    corrections: Buffer,
    /// Flat adjacency: for particle `i`, `neighbours[starts[i]..starts[i+1]]`.
    neighbour_starts: Buffer,
    neighbours: Buffer,
    scratch: SortScratch,

    table_size: CollisionTableSize,
    capacity: ParticleCapacity,
    /// Particles closer than twice this are pushed apart. Half the fabric
    /// thickness, so two layers rest one thickness apart.
    pub radius: f32,
    /// Grid spacing. Equal to the interaction diameter, so that the 27
    /// neighbouring buckets are guaranteed to hold every particle in range.
    spacing: f32,
    pub enabled: bool,
    /// Whether a grid has been built at least once.
    built: bool,
}

impl SelfCollision {
    /// Build the adjacency the constructor wants from a mesh's edges.
    pub fn adjacency(particle_count: ParticleInputCount, edges: &[[u32; 2]]) -> Vec<Vec<u32>> {
        let mut adjacency = vec![Vec::new(); usize::from(particle_count)];
        for &[a, b] in edges {
            adjacency[a as usize].push(b);
            adjacency[b as usize].push(a);
        }
        adjacency
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.spacing = radius * 2.0;
        self.built = false;
    }

    /// Record one self-collision pass.
    ///
    /// Rebuild after integration or constraint corrections: even a small
    /// displacement can cross a hash cell boundary. Reuse is valid only when
    /// every particle remains in its previously hashed cell. Cloth rebuilds
    /// before and after its structural solve to satisfy that requirement.
    pub fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        rebuild: bool,
    ) -> std::result::Result<(), SelfCollisionRecordError> {
        if !self.enabled {
            return Ok(());
        }
        let count = particles.count();
        if count < ParticleCount::from(2) {
            return Ok(());
        }
        if count > self.capacity.count() {
            return Err(SelfCollisionRecordError::Capacity {
                capacity: self.capacity,
                count,
            });
        }

        let inverse_spacing = 1.0 / self.spacing;

        if !rebuild && !self.built {
            // Nothing to collide against yet: the first substep after a
            // rebuild-less start still has to build one.
            return self.record(batch, particles, true);
        }
        if !rebuild {
            return self.record_collide(batch, particles, inverse_spacing);
        }
        self.built = true;

        let mut hash_parameters = PassParameters::new();
        hash_parameters.insert("positions".into(), (particles.positions.clone()).into());
        hash_parameters.insert("cells".into(), (self.cells.clone()).into());
        hash_parameters.insert("indices".into(), (self.indices.clone()).into());
        count.bind(&mut hash_parameters);
        self.table_size.bind(&mut hash_parameters);
        hash_parameters.insert("inverse_spacing".into(), (inverse_spacing).into());
        hash_parameters.insert("pad".into(), (0u32).into());
        SelfCollisionKernel::Hash.dispatch(
            batch,
            &self.hash,
            &hash_parameters,
            count.invocations(),
        )?;

        // Sorting the indices by bucket is what makes a bucket contiguous.
        self.sort
            .record(
                batch,
                &self.cells,
                &self.indices,
                &mut self.scratch,
                count.sort_items(),
                32.into(),
            )
            .map_err(SelfCollisionRecordError::Sort)?;

        let mut clear_parameters = PassParameters::new();
        clear_parameters.insert("starts".into(), (self.starts.clone()).into());
        self.table_size.bind(&mut clear_parameters);
        count.bind(&mut clear_parameters);
        clear_parameters.insert("pad0".into(), (0u32).into());
        clear_parameters.insert("pad1".into(), (0u32).into());
        SelfCollisionKernel::ClearRanges.dispatch(
            batch,
            &self.clear_ranges,
            &clear_parameters,
            self.table_size.clear_invocations(),
        )?;

        let mut range_parameters = PassParameters::new();
        range_parameters.insert("cells".into(), (self.cells.clone()).into());
        range_parameters.insert("starts".into(), (self.starts.clone()).into());
        count.bind(&mut range_parameters);
        self.table_size.bind(&mut range_parameters);
        range_parameters.insert("pad0".into(), (0u32).into());
        range_parameters.insert("pad1".into(), (0u32).into());
        SelfCollisionKernel::CellRanges.dispatch(
            batch,
            &self.cell_ranges,
            &range_parameters,
            count.invocations(),
        )?;

        self.record_collide(batch, particles, inverse_spacing)
    }

    /// The collision itself: two dispatches against whatever grid is current.
    fn record_collide(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        inverse_spacing: f32,
    ) -> std::result::Result<(), SelfCollisionRecordError> {
        let count = particles.count();
        let mut collide_parameters = PassParameters::new();
        collide_parameters.insert("positions".into(), (particles.positions.clone()).into());
        collide_parameters.insert("cells".into(), (self.cells.clone()).into());
        collide_parameters.insert("sorted".into(), (self.indices.clone()).into());
        collide_parameters.insert("starts".into(), (self.starts.clone()).into());
        collide_parameters.insert("corrections".into(), (self.corrections.clone()).into());
        collide_parameters.insert(
            "neighbour_starts".into(),
            (self.neighbour_starts.clone()).into(),
        );
        collide_parameters.insert("neighbours".into(), (self.neighbours.clone()).into());
        count.bind(&mut collide_parameters);
        self.table_size.bind(&mut collide_parameters);
        collide_parameters.insert("inverse_spacing".into(), (inverse_spacing).into());
        collide_parameters.insert("radius".into(), (self.radius).into());
        SelfCollisionKernel::Collide.dispatch(
            batch,
            &self.collide,
            &collide_parameters,
            count.invocations(),
        )?;

        let mut apply_parameters = PassParameters::new();
        apply_parameters.insert("positions".into(), (particles.positions.clone()).into());
        apply_parameters.insert("corrections".into(), (self.corrections.clone()).into());
        count.bind(&mut apply_parameters);
        apply_parameters.insert("pad0".into(), (0u32).into());
        apply_parameters.insert("pad1".into(), (0u32).into());
        apply_parameters.insert("pad2".into(), (0u32).into());
        SelfCollisionKernel::Apply.dispatch(
            batch,
            &self.apply,
            &apply_parameters,
            count.invocations(),
        )?;

        Ok(())
    }
}

impl SubstepHook for SelfCollision {
    type Error = SelfCollisionRecordError;
    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        _substep: SubstepDuration,
    ) -> std::result::Result<(), Self::Error> {
        // Integration can move particles across cell boundaries.
        SelfCollision::record(self, batch, particles, true)
    }
}

#[cfg(test)]
mod tests;
