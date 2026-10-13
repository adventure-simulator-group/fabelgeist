//! Dispatch and buffer orchestration for surface extraction.
use super::DualContouringDefinition;
use crate::Scan;
use crate::gather::Gather;
use anyhow::Result;
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferDefinition};
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};

use fabelgeist_gpu::data::PassParameter;
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
        let mut params = PassParameters::new();
        params.insert("grid_x", grid.0);
        params.insert("grid_y", grid.1);
        params.insert("grid_z", grid.2);
        params.insert("max_vertices", max_vertices);
        params.insert("max_indices", max_indices);
        params.insert("front_mode", u32::from(front_mode));
        params.insert("threshold", threshold);
        params.insert("scale_x", scale.0);
        params.insert("scale_y", scale.1);
        params.insert("scale_z", scale.2);
        params.insert("offset_x", offset.0);
        params.insert("offset_y", offset.1);
        params.insert("offset_z", offset.2);

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
        let mut sync_params = PassParameters::new();
        sync_params.insert("inclusive_offsets", index_inclusive_offsets.clone());
        sync_params.insert("indirect", output_indirect.clone());
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

        // 8. Deinterleave positions and normals
        let out_positions = Buffer::new(
            context,
            (max_vertices as u64 * 12).into(), // vec3 pos
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dual_contouring_positions").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::Vertex),
        )?;
        let out_normals = Buffer::new(
            context,
            (max_vertices as u64 * 12).into(), // vec3 norm
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("dual_contouring_normals").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::Vertex),
        )?;

        let mut deinterleave_params = PassParameters::new();
        deinterleave_params.insert("vertices", PassParameter::from(output_vertices));
        deinterleave_params.insert("out_positions", PassParameter::from(out_positions.clone()));
        deinterleave_params.insert("out_normals", PassParameter::from(out_normals.clone()));

        let workgroups_x = max_vertices.div_ceil(64);
        fabelgeist_gpu::data::gpu::ComputePass::dispatch(
            context,
            definition.deinterleave_pipeline.clone(),
            deinterleave_params,
            [workgroups_x, 1, 1].into(),
        )?;

        Ok((out_positions, out_normals, output_indices, output_indirect))
    }
}
