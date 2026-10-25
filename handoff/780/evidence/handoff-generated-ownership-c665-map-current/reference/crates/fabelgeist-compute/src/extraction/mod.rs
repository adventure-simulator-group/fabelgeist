//! Native lattice, prefix and output bounds for surface extraction.

use crate::ScanElementCount;
use fabelgeist_gpu::prelude::{BufferByteLength, BufferElementStride, PassParameter, PassParameters};
use fabelgeist_mesh::MeshVertexCount;
use fabelgeist_gpu::data::gpu::resource::{GpuResource, ResourceType};
use std::collections::TryReserveError;
use std::num::{NonZeroU32, TryFromIntError};

pub use output::{ExtractedVertices, IndexedExtraction, TriangleExtraction};
pub(crate) use projection::AttributeLabels;

mod output;
mod projection;
pub(crate) mod wgsl;

// Built-in active cells use [1, axis - 2); transverse quad cells use
// [2, axis - 2). These are the total excluded samples in the native bounds.
const ACTIVE_CELL_AXIS_INSET: u32 = 3;
const QUAD_CROSS_AXIS_INSET: u32 = 4;

/// A nonempty lattice whose linear native sample addresses fit in a GPU word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtractionGrid {
    axes: [u32; 3],
    population: ScanElementCount,
}

/// Arbitrary index slots, independently of the number of complete primitives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtractionIndexCapacity(NonZeroU32);

#[derive(Clone, Copy, Debug)]
pub(crate) struct VertexOutputLayout {
    vertices: MeshVertexCount,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum EmissionKind {
    TriangleVertices,
    QuadIndices,
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("extraction lattice axes must be nonzero: {axes:?}")]
    EmptyGrid { axes: [u32; 3] },
    #[error("extraction needs a 3D distance-field texture, received {actual:?}")]
    SourceKind { actual: ResourceType },
    #[error("extraction lattice {expected:?} differs from distance-field extent {actual:?}")]
    SourceExtent {
        expected: wgpu::Extent3d,
        actual: wgpu::Extent3d,
    },
    #[error("extraction lattice volume overflows widened arithmetic: {axes:?}")]
    GridVolume { axes: [u32; 3] },
    #[error("extraction lattice population exceeds native addressing")]
    GridPopulation(#[source] TryFromIntError),
    #[error("extraction vertex prefix exceeds native addressing")]
    VertexPrefix(#[source] TryFromIntError),
    #[error("extraction index prefix exceeds native addressing")]
    IndexPrefix(#[source] TryFromIntError),
    #[error("extraction vertex output capacity must be nonzero")]
    EmptyVertices,
    #[error("extraction index output capacity must be nonzero")]
    EmptyIndices,
    #[error("failed to allocate {population:?} extraction cell-ID words")]
    CellIds {
        population: ScanElementCount,
        #[source]
        source: TryReserveError,
    },
    #[error("extraction packed vertex attributes exceed native addressing")]
    PackedAttributes(#[source] TryFromIntError),
    #[error("extraction vertex storage needs {expected} bytes, received {actual}")]
    VertexStorage {
        expected: BufferByteLength,
        actual: BufferByteLength,
    },
}

impl TryFrom<[u32; 3]> for ExtractionGrid {
    type Error = ExtractionError;

    fn try_from(axes: [u32; 3]) -> Result<Self, Self::Error> {
        if axes.contains(&0) {
            return Err(ExtractionError::EmptyGrid { axes });
        }
        let volume = axes
            .into_iter()
            .try_fold(1u64, |volume, axis| volume.checked_mul(u64::from(axis)))
            .ok_or(ExtractionError::GridVolume { axes })?;
        let population = u32::try_from(volume).map_err(ExtractionError::GridPopulation)?;
        Ok(Self {
            axes,
            population: ScanElementCount::from_elements(population),
        })
    }
}

impl ExtractionGrid {
    pub(crate) fn ensure_sdf(self, source: &GpuResource) -> Result<(), ExtractionError> {
        let GpuResource::Texture3d(texture) = source else {
            return Err(ExtractionError::SourceKind {
                actual: source.resource_type(),
            });
        };
        let [width, height, depth_or_array_layers] = self.axes;
        let expected = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        };
        let actual = texture.extent();
        if actual != expected {
            return Err(ExtractionError::SourceExtent { expected, actual });
        }
        Ok(())
    }

    pub(crate) fn word_bytes(self) -> BufferByteLength {
        self.population.byte_length()
    }

    /// Native upload payload: one unwritten-ID sentinel per lattice sample.
    pub(crate) fn initial_cell_ids(self) -> Result<Vec<u32>, ExtractionError> {
        let population = self.population;
        let slots = population.elements() as usize;
        let mut ids = Vec::new();
        ids.try_reserve_exact(slots)
            .map_err(|source| ExtractionError::CellIds { population, source })?;
        ids.resize(slots, u32::MAX);
        Ok(ids)
    }

    pub(crate) fn insert(self, parameters: &mut PassParameters) {
        let [x, y, z] = self.axes;
        parameters.insert("grid_x", x);
        parameters.insert("grid_y", y);
        parameters.insert("grid_z", z);
    }

    pub(crate) fn admit_triangle_prefix(self) -> Result<(), ExtractionError> {
        let active = self.axes
            .map(|axis| u64::from(axis.saturating_sub(ACTIVE_CELL_AXIS_INSET)));
        let cells = active.into_iter().product::<u64>();
        // The immutable built-in count program reads this exact native table.
        let per_cell = crate::marching_cubes::tables::TRI_COUNT_TABLE
            .iter()
            .copied()
            .fold(0, u32::max);
        u32::try_from(cells * u64::from(per_cell))
            .map_err(ExtractionError::VertexPrefix)?;
        Ok(())
    }

    pub(crate) fn admit_contour_prefix(self) -> Result<(), ExtractionError> {
        let [x, y, z] = self.axes
            .map(|axis| u64::from(axis.saturating_sub(ACTIVE_CELL_AXIS_INSET)));
        u32::try_from(x * y * z).map_err(ExtractionError::VertexPrefix)?;
        let [inner_x, inner_y, inner_z] = self.axes
            .map(|axis| u64::from(axis.saturating_sub(QUAD_CROSS_AXIS_INSET)));
        let quads = x * inner_y * inner_z
            + inner_x * y * inner_z
            + inner_x * inner_y * z;
        // Each accepted sign-changing edge emits exactly two triangles.
        u32::try_from(quads * u64::from(EmissionKind::QuadIndices.records_per_primitive()))
            .map_err(ExtractionError::IndexPrefix)?;
        Ok(())
    }
}

impl TryFrom<u32> for ExtractionIndexCapacity {
    type Error = ExtractionError;

    fn try_from(slots: u32) -> Result<Self, Self::Error> {
        NonZeroU32::new(slots)
            .map(Self)
            .ok_or(ExtractionError::EmptyIndices)
    }
}

impl ExtractionIndexCapacity {
    /// The advancing-front facade applies one authored budget to both roles.
    pub(crate) fn from_vertex_budget(vertices: MeshVertexCount) -> Result<Self, ExtractionError> {
        Self::try_from(vertices.native_elements())
    }

    pub(crate) fn byte_length(self) -> BufferByteLength {
        (u64::from(self.0.get()) * std::mem::size_of::<u32>() as u64).into()
    }
}

impl From<ExtractionIndexCapacity> for PassParameter {
    fn from(capacity: ExtractionIndexCapacity) -> Self {
        Self::Unsigned(capacity.0.get())
    }
}

impl VertexOutputLayout {
    pub(crate) fn new(vertices: MeshVertexCount) -> Result<Self, ExtractionError> {
        if vertices.is_empty() {
            return Err(ExtractionError::EmptyVertices);
        }
        // Deinterleave writes three packed f32 values per vertex, addressed by
        // the native u32 expression idx * 3 + component.
        u32::try_from(u64::from(vertices.native_elements()) * 3)
            .map_err(ExtractionError::PackedAttributes)?;
        Ok(Self { vertices })
    }

    pub(crate) fn storage_bytes(self, stride: BufferElementStride) -> BufferByteLength {
        (u64::from(self.vertices.native_elements()) * u64::from(stride.byte_length())).into()
    }

    pub(crate) fn packed_attribute_bytes(self) -> BufferByteLength {
        (u64::from(self.vertices.native_elements()) * 3 * std::mem::size_of::<f32>() as u64).into()
    }
}

impl EmissionKind {
    pub(crate) const fn records_per_primitive(self) -> u32 {
        match self {
            Self::TriangleVertices => 3,
            Self::QuadIndices => 6,
        }
    }

    pub(crate) fn indirect_bytes(self) -> BufferByteLength {
        let words = match self {
            Self::TriangleVertices => 4u64,
            Self::QuadIndices => 5u64,
        };
        (words * std::mem::size_of::<u32>() as u64).into()
    }
}
