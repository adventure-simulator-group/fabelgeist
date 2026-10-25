//! Cloth against itself, through a uniform spatial hash.
//!
//! Rebuilt from scratch every substep: hash each particle into a bucket, sort
//! by bucket, record where each bucket starts, then have every particle look
//! at the 27 buckets around it. A hierarchy would be the wrong tool -- the
//! particles are all the same size and roughly evenly spread, which is the one
//! case a grid wins outright.

use fabelgeist_gpu::prelude::BufferUpload;
use std::sync::Arc;

use anyhow::anyhow;
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_xpbd::{ParticleCount, Particles, SubstepHook};

use crate::wgsl;
use layout::{CollisionAdjacency, CollisionBuckets};
pub use layout::CollisionLayoutError;

mod layout;

/// Whether to rebuild cell membership or reuse a previously recorded grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridUpdate {
    Rebuild,
    /// Valid only while all particles remain in their previously hashed cells.
    /// A missing grid or changed population still requires a rebuild.
    Reuse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GridState {
    Unbuilt,
    Recorded(ParticleCount),
}

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

    table_size: CollisionBuckets,
    capacity: ParticleCount,
    /// Particles closer than twice this are pushed apart. Half the fabric
    /// thickness, so two layers rest one thickness apart.
    radius: f32,
    pub enabled: bool,
    /// Whether a grid has been recorded since the last radius change.
    grid_state: GridState,
}

impl SelfCollision {
    /// `adjacency` is, per particle, the particles it already shares a mesh
    /// edge with. Those pairs are skipped: a stretch constraint is already
    /// holding them at the right distance, and pushing them apart as well
    /// would inflate the fabric.
    pub fn new(
        context: &WgpuContext,
        cache: &KernelCache,
        particle_count: ParticleCount,
        adjacency: &[Vec<u32>],
        radius: f32,
    ) -> Result<Self> {
        if adjacency.len() != usize::from(particle_count) {
            return Err(anyhow!(
                "SelfCollision: {particle_count} particles but {} adjacency lists",
                adjacency.len()
            ));
        }
        let capacity = particle_count.max(ParticleCount::from(1));
        // Roughly two buckets per particle keeps the occupancy low enough that
        // a bucket is a handful of entries, and the memory is trivial.
        let table_size = CollisionBuckets::for_particles(capacity)?;

        let adjacency = CollisionAdjacency::new(adjacency)?;

        let storage = BufferDefinition::storage();
        Ok(Self {
            hash: cache.get(context, wgsl::HASH)?,
            clear_ranges: cache.get(context, wgsl::CLEAR_RANGES)?,
            cell_ranges: cache.get(context, wgsl::CELL_RANGES)?,
            collide: cache.get(context, wgsl::SELF_COLLIDE)?,
            apply: cache.get(context, wgsl::APPLY_CORRECTIONS)?,
            sort: RadixSort::with_cache(context, cache)?,

            cells: Buffer::new(
                context,
                (u64::from(capacity.native_elements()) * 4).into(),
                storage.clone().with_label(("self-collision cells").into()),
            )?,
            indices: Buffer::new(
                context,
                (u64::from(capacity.native_elements()) * 4).into(),
                storage
                    .clone()
                    .with_label(("self-collision indices").into()),
            )?,
            starts: Buffer::new(
                context,
                table_size.range_bytes(),
                storage
                    .clone()
                    .with_label(("self-collision buckets").into()),
            )?,
            corrections: Buffer::new(
                context,
                (u64::from(capacity.native_elements()) * 16).into(),
                storage
                    .clone()
                    .with_label(("self-collision corrections").into()),
            )?,
            neighbour_starts: Buffer::from_upload(
                context,
                BufferUpload::from_elements(adjacency.starts()),
                storage
                    .clone()
                    .with_label(("self-collision adjacency starts").into()),
            )?,
            neighbours: Buffer::from_upload(
                context,
                BufferUpload::from_elements(adjacency.neighbours()),
                storage.with_label(("self-collision adjacency").into()),
            )?,
            scratch: SortScratch::new(context, capacity.into())?,

            table_size,
            capacity,
            radius,
            enabled: true,
            grid_state: GridState::Unbuilt,
        })
    }

    /// Build the adjacency the constructor wants from a mesh's edges.
    pub fn adjacency(particle_count: usize, edges: &[[u32; 2]]) -> Vec<Vec<u32>> {
        let mut adjacency = vec![Vec::new(); particle_count];
        for &[a, b] in edges {
            adjacency[a as usize].push(b);
            adjacency[b as usize].push(a);
        }
        adjacency
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.grid_state = GridState::Unbuilt;
    }

    /// Record one self-collision pass.
    ///
    /// Rebuild after integration or constraint corrections: even a small
    /// displacement can cross a hash cell boundary. Reuse is valid only when
    /// every particle remains in its previously hashed cell. A changed live
    /// population always rebuilds. Cloth rebuilds before and after its
    /// structural solve to satisfy the spatial requirement.
    pub fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        update: GridUpdate,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let count = particles.count();
        if count < ParticleCount::from(2) {
            return Ok(());
        }
        if count > self.capacity {
            return Err(anyhow!(
                "SelfCollision: built for {} particles, given {count}",
                self.capacity
            ));
        }

        // Cell spacing is the interaction diameter, so the 27 neighboring
        // cells cover every candidate within range.
        let inverse_spacing = 1.0 / (self.radius * 2.0);

        if update == GridUpdate::Reuse {
            if self.grid_state == GridState::Recorded(count) {
                return self.record_collide(batch, particles, inverse_spacing);
            }
            // No grid, or another population: old sorted slots cannot be reused.
            return self.record(batch, particles, GridUpdate::Rebuild);
        }
        self.grid_state = GridState::Recorded(count);

        let mut hash_parameters = PassParameters::new();
        hash_parameters.insert("positions", particles.positions().clone());
        hash_parameters.insert("cells", self.cells.clone());
        hash_parameters.insert("indices", self.indices.clone());
        hash_parameters.insert("count", count);
        hash_parameters.insert("table_size", self.table_size);
        hash_parameters.insert("inverse_spacing", inverse_spacing);
        hash_parameters.insert("pad", 0u32);
        batch.dispatch_items(&self.hash, &hash_parameters, count.into())?;

        // Sorting the indices by bucket is what makes a bucket contiguous.
        self.sort.record(
            batch,
            &self.cells,
            &self.indices,
            &mut self.scratch,
            count.into(),
            32.into(),
        )?;

        let mut clear_parameters = PassParameters::new();
        clear_parameters.insert("starts", self.starts.clone());
        clear_parameters.insert("table_size", self.table_size);
        clear_parameters.insert("count", count);
        clear_parameters.insert("pad0", 0u32);
        clear_parameters.insert("pad1", 0u32);
        batch.dispatch_items(&self.clear_ranges, &clear_parameters, self.table_size.boundary_invocations())?;

        let mut range_parameters = PassParameters::new();
        range_parameters.insert("cells", self.cells.clone());
        range_parameters.insert("starts", self.starts.clone());
        range_parameters.insert("count", count);
        range_parameters.insert("table_size", self.table_size);
        range_parameters.insert("pad0", 0u32);
        range_parameters.insert("pad1", 0u32);
        batch.dispatch_items(&self.cell_ranges, &range_parameters, count.into())?;

        self.record_collide(batch, particles, inverse_spacing)
    }

    /// The collision itself: two dispatches against whatever grid is current.
    fn record_collide(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        inverse_spacing: f32,
    ) -> Result<()> {
        let count = particles.count();
        let mut collide_parameters = PassParameters::new();
        collide_parameters.insert("positions", particles.positions().clone());
        collide_parameters.insert("cells", self.cells.clone());
        collide_parameters.insert("sorted", self.indices.clone());
        collide_parameters.insert("starts", self.starts.clone());
        collide_parameters.insert("corrections", self.corrections.clone());
        collide_parameters.insert("neighbour_starts", self.neighbour_starts.clone());
        collide_parameters.insert("neighbours", self.neighbours.clone());
        collide_parameters.insert("count", count);
        collide_parameters.insert("table_size", self.table_size);
        collide_parameters.insert("inverse_spacing", inverse_spacing);
        collide_parameters.insert("radius", self.radius);
        batch.dispatch_items(&self.collide, &collide_parameters, count.into())?;

        let mut apply_parameters = PassParameters::new();
        apply_parameters.insert("positions", particles.positions().clone());
        apply_parameters.insert("corrections", self.corrections.clone());
        apply_parameters.insert("count", count);
        apply_parameters.insert("pad0", 0u32);
        apply_parameters.insert("pad1", 0u32);
        apply_parameters.insert("pad2", 0u32);
        batch.dispatch_items(&self.apply, &apply_parameters, count.into())?;

        Ok(())
    }
}

impl SubstepHook for SelfCollision {
    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        _substep: f32,
    ) -> Result<()> {
        // Integration can move particles across cell boundaries.
        SelfCollision::record(self, batch, particles, GridUpdate::Rebuild)
    }
}
