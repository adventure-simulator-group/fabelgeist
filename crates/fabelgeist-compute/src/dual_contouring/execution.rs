//! Dispatch and buffer orchestration for surface extraction.

use crate::extraction::{AttributeLabels, EmissionKind, VertexOutputLayout};
use crate::{ExtractionGrid, ExtractionIndexCapacity, IndexedExtraction};
use fabelgeist_mesh::MeshVertexCount;
use super::DualContouringDefinition;
use crate::Scan;
use crate::gather::Gather;
use anyhow::Result;
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferDefinition};
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};

use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_gpu::data::gpu::resource::GpuResource;

pub struct DualContouring;

#[derive(Clone, Copy, Debug)]
enum FrontMode {
    Fixed,
    ShortestDiagonal,
}

impl FrontMode {
    fn native_word(self) -> u32 {
        match self {
            Self::Fixed => 0,
            Self::ShortestDiagonal => 1,
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "surface extraction passes grid, threshold, output bounds and world transform to the GPU"
)]
impl DualContouring {
    /// Emit complete quads whose four vertices fit the vertex budget.
    /// The index budget can contain any positive number of slots; the
    /// indirect count includes only fully written quads. The grid must match
    /// the distance-field texture.
    pub fn execute(
        context: &WgpuContext,
        definition: &DualContouringDefinition,
        sdf: &GpuResource,
        grid: ExtractionGrid,
        threshold: f32,
        max_vertices: MeshVertexCount,
        max_indices: ExtractionIndexCapacity,
        scale: (f32, f32, f32),
        offset: (f32, f32, f32),
    ) -> Result<IndexedExtraction> {
        Self::execute_internal(
            context,
            definition,
            sdf,
            grid,
            threshold,
            max_vertices,
            max_indices,
            scale,
            offset,
            FrontMode::Fixed,
        )
    }

    pub(crate) fn execute_advancing_front(
        context: &WgpuContext,
        definition: &DualContouringDefinition,
        sdf: &GpuResource,
        grid: ExtractionGrid,
        threshold: f32,
        max_vertices: MeshVertexCount,
        max_indices: ExtractionIndexCapacity,
        scale: (f32, f32, f32),
        offset: (f32, f32, f32),
    ) -> Result<IndexedExtraction> {
        Self::execute_internal(
            context,
            definition,
            sdf,
            grid,
            threshold,
            max_vertices,
            max_indices,
            scale,
            offset,
            FrontMode::ShortestDiagonal,
        )
    }

    fn execute_internal(
        context: &WgpuContext,
        definition: &DualContouringDefinition,
        sdf: &GpuResource,
        grid: ExtractionGrid,
        threshold: f32,
        max_vertices: MeshVertexCount,
        max_indices: ExtractionIndexCapacity,
        scale: (f32, f32, f32),
        offset: (f32, f32, f32),
        front_mode: FrontMode,
    ) -> Result<IndexedExtraction> {
        grid.ensure_sdf(sdf)?;
        grid.admit_contour_prefix()?;
        let output_layout = VertexOutputLayout::new(max_vertices)?;

        // Create uniform parameter buffer info
        let mut params = PassParameters::new();
        grid.insert(&mut params);
        params.insert("max_vertices", max_vertices);
        params.insert("max_indices", max_indices);
        params.insert("front_mode", front_mode.native_word());
        params.insert("threshold", threshold);
        params.insert("scale_x", scale.0);
        params.insert("scale_y", scale.1);
        params.insert("scale_z", scale.2);
        params.insert("offset_x", offset.0);
        params.insert("offset_y", offset.1);
        params.insert("offset_z", offset.2);

        // Gather Vertex Count
        let vertex_counts_buffer = Buffer::new(
            context,
            grid.word_bytes(),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dual_contouring_vertex_counts").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination),
        )?;
        Gather::execute_with_parameters(
            context,
            &definition.vertex_count_def,
            sdf,
            &GpuResource::Buffer(vertex_counts_buffer.clone()),
            Some(params.clone()),
        )?;

        let vertex_inclusive_offsets =
            Scan::execute(context, &definition.scan_def, &vertex_counts_buffer)?;

        // Initialize cell_vertex_indices buffer to 0xFFFFFFFF
        let initial_cell_ids = grid.initial_cell_ids()?;
        let cell_vertex_indices = Buffer::from_upload(
            context,
            BufferUpload::from_elements(&initial_cell_ids),
            BufferDefinition::storage().with_label(("cell_vertex_indices").into()),
        )?;

        // Stream Vertices
        let output_vertices = Buffer::new(
            context,
            output_layout.storage_bytes(definition.deinterleave_pipeline.storage_stride()?),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dual_contouring_output_vertices").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination),
        )?;
        let mut vertex_gen_params = params.clone();
        vertex_gen_params.insert("cell_vertex_indices", cell_vertex_indices.clone());
        crate::Stream::execute(
            context,
            &definition.vertex_stream_def,
            sdf,
            &vertex_counts_buffer,
            &vertex_inclusive_offsets,
            &output_vertices,
            Some(vertex_gen_params),
        )?;

        // Gather Index Count
        let index_counts_buffer = Buffer::new(
            context,
            grid.word_bytes(),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dual_contouring_index_counts").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination),
        )?;
        let mut index_count_params = params.clone();
        index_count_params.insert("cell_vertex_indices", cell_vertex_indices.clone());
        Gather::execute_with_parameters(
            context,
            &definition.index_count_def,
            sdf,
            &GpuResource::Buffer(index_counts_buffer.clone()),
            Some(index_count_params),
        )?;

        let index_inclusive_offsets =
            Scan::execute(context, &definition.scan_def, &index_counts_buffer)?;

        // Sync Indirect Buffer (using index count)
        let dummy_in = Buffer::new(
            context,
            (4u64).into(),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dummy_in").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination),
        )?;
        let dummy_out = Buffer::new(
            context,
            (4u64).into(),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dummy_out").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination),
        )?;
        let output_indirect = Buffer::new(
            context,
            EmissionKind::QuadIndices.indirect_bytes(),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dual_contouring_indirect").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::Indirect),
        )?;
        let mut sync_params = PassParameters::new();
        sync_params.insert("inclusive_offsets", index_inclusive_offsets.clone());
        sync_params.insert("indirect", output_indirect.clone());
        sync_params.insert("capacity", max_indices);
        crate::Map::execute_with_parameters(
            context,
            &definition.sync_indirect_def,
            Some(&GpuResource::Buffer(dummy_in)),
            &GpuResource::Buffer(dummy_out),
            Some(sync_params),
        )?;

        // Stream Indices
        let output_indices = Buffer::new(
            context,
            max_indices.byte_length(),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dual_contouring_output_indices").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::Index),
        )?;
        let mut index_gen_params = params.clone();
        index_gen_params.insert("cell_vertex_indices", cell_vertex_indices.clone());
        index_gen_params.insert("front_vertices", output_vertices.clone());
        crate::Stream::execute(
            context,
            &definition.index_stream_def,
            sdf,
            &index_counts_buffer,
            &index_inclusive_offsets,
            &output_indices,
            Some(index_gen_params),
        )?;

        let attributes = output_layout.project(
            context,
            &definition.deinterleave_pipeline,
            output_vertices,
            AttributeLabels {
                positions: "dual_contouring_positions".into(),
                normals: "dual_contouring_normals".into(),
            },
        )?;

        Ok(IndexedExtraction::new(
            attributes,
            max_indices,
            output_indices,
            output_indirect,
        ))
    }
}
