//! Admit adjacency before compiling kernels and allocating the spatial hash.
use super::hash_layout::CollisionTableSize;
use super::{SelfCollision, SelfCollisionBuffer, SelfCollisionBuildError, SelfCollisionKernel};
use fabelgeist_compute::kernel::KernelCache;
use fabelgeist_compute::{RadixSort, SortScratch};
use fabelgeist_gpu::prelude::{BufferDefinition, BufferUpload, WgpuContext};
use fabelgeist_xpbd::{ParticleCapacity, ParticleCount, ParticleInputCount};

impl SelfCollision {
    /// Adjacency lists identify pairs already held together by a mesh edge.
    /// Admission retains the existing native count narrowing and sentinel layout.
    pub fn new(
        context: &WgpuContext,
        cache: &KernelCache,
        particle_count: ParticleCount,
        adjacency: &[Vec<u32>],
        radius: f32,
    ) -> Result<Self, SelfCollisionBuildError> {
        let lists = ParticleInputCount::from(adjacency.len());
        if lists.gpu_count() != particle_count {
            return Err(SelfCollisionBuildError::Adjacency {
                particles: particle_count,
                lists,
            });
        }
        let capacity = ParticleCapacity::from(particle_count);
        let table_size = CollisionTableSize::for_capacity(capacity);
        let mut starts_data = Vec::with_capacity(usize::from(capacity.count()) + 1);
        let mut flat = Vec::new();
        for list in adjacency {
            starts_data.push(flat.len() as u32);
            flat.extend_from_slice(list);
        }
        starts_data.push(flat.len() as u32);
        if flat.is_empty() {
            // Native allocation needs a sentinel; all adjacency ranges stay empty.
            flat.push(0);
        }
        let storage = BufferDefinition::storage();
        Ok(Self {
            hash: SelfCollisionKernel::Hash.load(context, cache)?,
            clear_ranges: SelfCollisionKernel::ClearRanges.load(context, cache)?,
            cell_ranges: SelfCollisionKernel::CellRanges.load(context, cache)?,
            collide: SelfCollisionKernel::Collide.load(context, cache)?,
            apply: SelfCollisionKernel::Apply.load(context, cache)?,
            sort: RadixSort::with_cache(context, cache).map_err(SelfCollisionBuildError::Sort)?,
            cells: SelfCollisionBuffer::Cells.allocate(
                context,
                capacity.sort_items().word_bytes(),
                storage.clone(),
            )?,
            indices: SelfCollisionBuffer::Indices.allocate(
                context,
                capacity.sort_items().word_bytes(),
                storage.clone(),
            )?,
            starts: SelfCollisionBuffer::BucketStarts.allocate(
                context,
                table_size.starts_bytes(),
                storage.clone(),
            )?,
            corrections: SelfCollisionBuffer::Corrections.allocate(
                context,
                capacity.record_bytes(),
                storage.clone(),
            )?,
            neighbour_starts: SelfCollisionBuffer::AdjacencyStarts.upload(
                context,
                BufferUpload::from_elements(&starts_data),
                storage.clone(),
            )?,
            neighbours: SelfCollisionBuffer::Adjacency.upload(
                context,
                BufferUpload::from_elements(&flat),
                storage,
            )?,
            scratch: SortScratch::new(context, capacity.sort_items())
                .map_err(SelfCollisionBuildError::Scratch)?,
            table_size,
            capacity,
            radius,
            spacing: radius * 2.0,
            enabled: true,
            built: false,
        })
    }
}
