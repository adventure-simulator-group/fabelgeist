//! Uniform-grid spatial hashing of spheres.
//!
//! Many solvers ask the same question of a cloud of spheres: *which of them are
//! near this one?* Cloth self-collision asks it of particles, a plant ecosystem
//! asks it of the bounding spheres of its branch modules. The answer is the
//! same structure every time: hash each sphere's centre into a grid cell, sort
//! by bucket, and remember where each bucket's run begins and ends. A query
//! then walks the 27 cells around a point.
//!
//! Two builds of it:
//!
//! * [`HashGrid`] -- on the host, with a counting sort. The oracle the GPU
//!   build is checked against, and the path for scenes too small to be worth a
//!   dispatch.
//! * [`gpu::GpuHashGrid`] -- hash, radix sort and bucket ranges as compute
//!   passes recorded into a caller's [`fabelgeist_compute::KernelBatch`]. Its query
//!   is published as WGSL source ([`gpu::HASH_WGSL`], [`gpu::RANGE_WGSL`]) so that a kernel can walk
//!   the grid itself, without a round trip.
//!
//! Both use the same hash, so a bucket index means the same thing on either
//! side. Distinct cells can share a bucket; a query rejects the entries whose
//! own cell is not the one it asked for, so a collision costs a comparison and
//! never a duplicate.
//!
//! [`overlap`] holds the sphere-sphere intersection volume, on the host and in
//! WGSL, because "how much do these spheres overlap" is what most callers do
//! with the neighbours they find.

pub use host::HashGrid;
pub use layout::{CellScale, HashBucketCount, HashLayout, HashLayoutError, HashPointCount};
pub use overlap::{SPHERE_OVERLAP_WGSL, sphere_overlap_volume, sphere_volume};

pub mod host;
mod layout;
pub mod overlap;

#[cfg(feature = "gpu")]
pub mod gpu;

/// Marks an empty bucket in the range table.
pub const EMPTY: u32 = u32::MAX;

/// The integer cell a point falls into, using the caller's coordinate units.
/// The host cast retains Rust's saturating float-to-integer behavior; GPU
/// producers must supply representable projected cells and query offsets.
#[inline]
pub fn cell_of(point: [f32; 3], scale: CellScale) -> [i32; 3] {
    let inverse_cell = scale.inverse_cell();
    [
        (point[0] * inverse_cell).floor() as i32,
        (point[1] * inverse_cell).floor() as i32,
        (point[2] * inverse_cell).floor() as i32,
    ]
}

/// The bucket a native integer cell hashes to in the admitted table.
///
/// Three large odd primes, the usual spatial hash; the WGSL in
/// [`gpu::HASH_WGSL`] is the same function word for word.
#[inline]
pub fn bucket_of(cell: [i32; 3], table_size: HashBucketCount) -> u32 {
    let h = (cell[0] as u32).wrapping_mul(73_856_093)
        ^ (cell[1] as u32).wrapping_mul(19_349_663)
        ^ (cell[2] as u32).wrapping_mul(83_492_791);
    h & table_size.mask()
}

