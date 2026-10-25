//! Primitive populations and the native node layouts derived from them.

use fabelgeist_compute::SortItemCount;
use fabelgeist_gpu::prelude::{Buffer, BufferByteLength, InvocationCount, PassParameter};
use fabelgeist_mesh::MeshTriangleCount;
use std::fmt;
use std::num::TryFromIntError;

// Nodes and primitive bounds each occupy two native vec4 records. Atomic
// scratch omits their tag words and keeps only lower/upper xyz.
const BOUNDS_PAIR_BYTES: u64 = 2 * 4 * size_of::<f32>() as u64;
const ATOMIC_BOUND_WORDS: u64 = 6;

/// Primitives ordered by the hierarchy, independently of their geometry kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct BvhPrimitiveCount(u32);

/// Nodes in a hierarchy or one of its contiguous node ranges.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct BvhNodeCount(u32);

/// A primitive population with representable node and atomic-word addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BvhLayout {
    primitives: BvhPrimitiveCount,
    nodes: BvhNodeCount,
}

/// The derived native layout cannot address the requested population.
#[derive(Debug)]
pub struct BvhLayoutError {
    primitives: BvhPrimitiveCount,
    source: TryFromIntError,
}

/// Primitive bounds do not contain the requested lower/upper native records.
#[derive(Clone, Copy, Debug)]
pub struct BvhBoundsError {
    primitives: BvhPrimitiveCount,
    available: BufferByteLength,
}

impl From<u32> for BvhPrimitiveCount {
    fn from(primitives: u32) -> Self {
        Self(primitives)
    }
}

impl TryFrom<usize> for BvhPrimitiveCount {
    type Error = TryFromIntError;

    fn try_from(primitives: usize) -> Result<Self, Self::Error> {
        u32::try_from(primitives).map(Self)
    }
}

impl From<MeshTriangleCount> for BvhPrimitiveCount {
    fn from(triangles: MeshTriangleCount) -> Self {
        Self(triangles.native_elements())
    }
}

impl BvhPrimitiveCount {
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(super) fn internal_nodes(self) -> BvhNodeCount {
        BvhNodeCount(self.0.saturating_sub(1))
    }

    pub(super) fn word_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.0) * size_of::<u32>() as u64)
    }

    pub(super) fn check_bounds(self, bounds: &Buffer) -> Result<(), BvhBoundsError> {
        let available = bounds.size.min(bounds.buffer.size().into());
        if available < self.bounds_bytes() {
            return Err(BvhBoundsError {
                primitives: self,
                available,
            });
        }
        Ok(())
    }

    /// Native lower/upper vec4 records, written by a primitive-bounds kernel.
    pub fn bounds_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.0) * BOUNDS_PAIR_BYTES)
    }
}

impl TryFrom<BvhPrimitiveCount> for BvhLayout {
    type Error = BvhLayoutError;

    fn try_from(primitives: BvhPrimitiveCount) -> Result<Self, Self::Error> {
        if primitives.is_empty() {
            return Ok(Self::EMPTY);
        }
        let nodes = u32::try_from(2 * u64::from(primitives.0) - 1)
            .map_err(|source| BvhLayoutError { primitives, source })?;
        // Every atomic load/store addresses index*6 + axis. Checking only
        // the node population would leave those native offsets unprotected.
        u32::try_from(u64::from(nodes) * ATOMIC_BOUND_WORDS)
            .map_err(|source| BvhLayoutError { primitives, source })?;
        Ok(Self { primitives, nodes: BvhNodeCount(nodes) })
    }
}

impl BvhLayout {
    pub const EMPTY: Self = Self {
        primitives: BvhPrimitiveCount(0),
        nodes: BvhNodeCount(0),
    };

    pub fn primitive_count(self) -> BvhPrimitiveCount {
        self.primitives
    }

    pub fn node_count(self) -> BvhNodeCount {
        self.nodes
    }

    pub(super) fn with_sentinel(self) -> Self {
        if self.primitives.is_empty() {
            Self { primitives: BvhPrimitiveCount(1), nodes: BvhNodeCount(1) }
        } else {
            self
        }
    }

    pub(super) fn node_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.nodes.0) * BOUNDS_PAIR_BYTES)
    }

    pub(super) fn parent_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.nodes.0) * size_of::<u32>() as u64)
    }

    pub(super) fn atomic_bounds_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.nodes.0) * ATOMIC_BOUND_WORDS * 4)
    }

    pub(super) fn scene_bounds_bytes() -> BufferByteLength {
        BufferByteLength::from(ATOMIC_BOUND_WORDS * 4)
    }
}

impl From<BvhPrimitiveCount> for PassParameter {
    fn from(primitives: BvhPrimitiveCount) -> Self {
        Self::Unsigned(primitives.0)
    }
}

impl From<BvhPrimitiveCount> for InvocationCount {
    fn from(primitives: BvhPrimitiveCount) -> Self {
        Self::from(primitives.0)
    }
}

impl From<BvhNodeCount> for InvocationCount {
    fn from(nodes: BvhNodeCount) -> Self {
        Self::from(nodes.0)
    }
}

impl From<BvhPrimitiveCount> for SortItemCount {
    fn from(primitives: BvhPrimitiveCount) -> Self {
        Self::from(primitives.0)
    }
}

impl From<BvhPrimitiveCount> for usize {
    fn from(primitives: BvhPrimitiveCount) -> Self {
        primitives.0 as usize
    }
}

impl From<BvhNodeCount> for usize {
    fn from(nodes: BvhNodeCount) -> Self {
        nodes.0 as usize
    }
}

impl fmt::Display for BvhPrimitiveCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for BvhNodeCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for BvhLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "BVH layout for {} primitives exceeds native addresses", self.primitives)
    }
}

impl std::error::Error for BvhLayoutError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl fmt::Display for BvhBoundsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "BVH bounds for {} primitives require {} bytes, but only {} are available",
            self.primitives,
            self.primitives.bounds_bytes(),
            self.available,
        )
    }
}

impl std::error::Error for BvhBoundsError {}
