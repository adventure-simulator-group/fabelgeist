//! Dispatch and buffer orchestration for surface extraction.
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

#[expect(
    clippy::too_many_arguments,
    reason = "surface extraction passes grid, threshold, output bounds and world transform to the GPU"
)]
impl DualContouring {
    pub fn execute(
        context: &WgpuContext,
        definition: &DualContouringDefinition,
        sdf: &GpuResource,
        grid: (u32, u32, u32),
        threshold: f32,
        max_vertices: u32,
        max_indices: u32,
        scale: (f32, f32, f32),
        offset: (f32, f32, f32),
    ) -> Result<(Buffer, Buffer, Buffer, Buffer)> {
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
            false,
        )
    }

    pub(crate) fn execute_advancing_front(
        context: &WgpuContext,
        definition: &DualContouringDefinition,
        sdf: &GpuResource,
        grid: (u32, u32, u32),
        threshold: f32,
        max_vertices: u32,
        max_indices: u32,
        scale: (f32, f32, f32),
        offset: (f32, f32, f32),
    ) -> Result<(Buffer, Buffer, Buffer, Buffer)> {
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
            true,
        )
    }

    fn execute_internal(
        context: &WgpuContext,
        definition: &DualContouringDefinition,
        sdf: &GpuResource,
        grid: (u32, u32, u32),
        threshold: f32,
        max_vertices: u32,
        max_indices: u32,
        scale: (f32, f32, f32),
        offset: (f32, f32, f32),
        front_mode: bool,
    ) -> Result<(Buffer, Buffer, Buffer, Buffer)> {
        let grid_total = (grid.0 * grid.1 * grid.2) as u64;

        // Create uniform parameter buffer info
        let params = PassParameters::from([
            ("grid_x".into(), (grid.0).into()),
            ("grid_y".into(), (grid.1).into()),
            ("grid_z".into(), (grid.2).into()),
            ("max_vertices".into(), (max_vertices).into()),
            ("max_indices".into(), (max_indices).into()),
            ("front_mode".into(), (u32::from(front_mode)).into()),
            ("threshold".into(), (threshold).into()),
            ("scale_x".into(), (scale.0).into()),
            ("scale_y".into(), (scale.1).into()),
            ("scale_z".into(), (scale.2).into()),
            ("offset_x".into(), (offset.0).into()),
            ("offset_y".into(), (offset.1).into()),
            ("offset_z".into(), (offset.2).into()),
        ]);

        // 1. Gather Vertex Count
        let vertex_counts_buffer = Buffer::new(
            context,
            (grid_total * 4).into(),
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

        // 2. Gather Index Count
        let index_counts_buffer = Buffer::new(
            context,
            (grid_total * 4).into(),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dual_contouring_index_counts").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination),
        )?;
        Gather::execute_with_parameters(
            context,
            &definition.index_count_def,
            sdf,
            &GpuResource::Buffer(index_counts_buffer.clone()),
            Some(params.clone()),
        )?;

        // 3. Scan Passes
        let vertex_inclusive_offsets =
            Scan::execute(context, &definition.scan_def, &vertex_counts_buffer)?;
        let index_inclusive_offsets =
            Scan::execute(context, &definition.scan_def, &index_counts_buffer)?;

        // 4. Sync Indirect Buffer (using index count)
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
            (20u64).into(), // 5 * u32 for DrawIndexedIndirect
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dual_contouring_indirect").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::Indirect),
        )?;
        let sync_params = PassParameters::from([
            (
                "inclusive_offsets".into(),
                (index_inclusive_offsets.clone()).into(),
            ),
            ("indirect".into(), (output_indirect.clone()).into()),
        ]);
        crate::Map::execute_with_parameters(
            context,
            &definition.sync_indirect_def,
            Some(&GpuResource::Buffer(dummy_in)),
            &GpuResource::Buffer(dummy_out),
            Some(sync_params),
        )?;

        // 5. Initialize cell_vertex_indices buffer to 0xFFFFFFFF
        let cell_vertex_indices = Buffer::from_upload(
            context,
            BufferUpload::from_elements(&vec![0xFFFFFFFFu32; grid_total as usize]),
            BufferDefinition::storage().with_label(("cell_vertex_indices").into()),
        )?;

        // 6. Stream Vertices
        let output_vertices = Buffer::new(
            context,
            (max_vertices as u64 * 32).into(), // vec4 pos + vec4 norm
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dual_contouring_output_vertices").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination),
        )?;
        let mut vertex_gen_params = params.clone();
        vertex_gen_params.insert(
            "cell_vertex_indices".into(),
            (cell_vertex_indices.clone()).into(),
        );
        crate::Stream::execute(
            context,
            &definition.vertex_stream_def,
            sdf,
            &vertex_counts_buffer,
            &vertex_inclusive_offsets,
            &output_vertices,
            Some(vertex_gen_params),
        )?;

        // 7. Stream Indices
        let output_indices = Buffer::new(
            context,
            (max_indices as u64 * 4).into(),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dual_contouring_output_indices").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::Index),
        )?;
        let mut index_gen_params = params.clone();
        index_gen_params.overlay(PassParameters::from([
            (
                "cell_vertex_indices".into(),
                (cell_vertex_indices.clone()).into(),
            ),
            ("front_vertices".into(), (output_vertices.clone()).into()),
        ]));
        crate::Stream::execute(
            context,
            &definition.index_stream_def,
            sdf,
            &index_counts_buffer,
            &index_inclusive_offsets,
            &output_indices,
            Some(index_gen_params),
        )?;

        // 8. Deinterleave positions and normals
        let attributes = crate::surface_attributes::SurfaceAttributes::new(
            context,
            &definition.deinterleave_pipeline,
            output_vertices,
            fabelgeist_gpu::prelude::BufferByteLength::from(max_vertices as u64 * 12),
            fabelgeist_gpu::prelude::WorkgroupGrid::from([max_vertices.div_ceil(64), 1, 1]),
            crate::surface_attributes::SurfaceExtraction::DualContouring,
        )?;
        Ok((
            attributes.positions,
            attributes.normals,
            output_indices,
            output_indirect,
        ))
    }
}
