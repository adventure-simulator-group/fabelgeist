use crate::extraction::wgsl as shared_wgsl;
use crate::extraction::{AttributeLabels, EmissionKind, VertexOutputLayout};
use crate::gather::Gather;
use crate::{
    ExtractionGrid, GatherDefinition, MapDefinition, Scan, ScanDefinition, StreamDefinition,
    TriangleExtraction,
};
use anyhow::Result;
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferDefinition};
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};
use fabelgeist_mesh::MeshVertexCount;

pub mod tables;
mod wgsl;

#[derive(Clone, Debug)]
pub struct MarchingCubesDefinition {
    count_def: GatherDefinition,
    scan_def: ScanDefinition,
    stream_def: StreamDefinition,
    sync_indirect_def: MapDefinition,
    tri_table_buffer: Buffer,
    edge_table_buffer: Buffer,
    tri_count_table_buffer: Buffer,
    deinterleave_pipeline: crate::ResourcePipeline,
}

pub struct MarchingCubes;

impl PartialEq for MarchingCubesDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.count_def == other.count_def
            && self.scan_def == other.scan_def
            && self.stream_def == other.stream_def
            && self.sync_indirect_def == other.sync_indirect_def
            && self.tri_table_buffer == other.tri_table_buffer
            && self.edge_table_buffer == other.edge_table_buffer
            && self.tri_count_table_buffer == other.tri_count_table_buffer
    }
}

impl MarchingCubesDefinition {
    pub fn new(context: &WgpuContext) -> Result<Self> {
        let tri_table_buffer = Buffer::from_upload(
            context,
            BufferUpload::from_elements(&tables::TRI_TABLE),
            BufferDefinition::storage().with_label(("TRI_TABLE").into()),
        )?;
        let edge_table_buffer = Buffer::from_upload(
            context,
            BufferUpload::from_elements(&tables::EDGE_TABLE),
            BufferDefinition::storage().with_label(("EDGE_TABLE").into()),
        )?;
        let tri_count_table_buffer = Buffer::from_upload(
            context,
            BufferUpload::from_elements(&tables::TRI_COUNT_TABLE),
            BufferDefinition::storage().with_label(("TRI_COUNT_TABLE").into()),
        )?;

        let count_wgsl = wgsl::count();

        let count_def = GatherDefinition::new(context, count_wgsl)?;
        let scan_def = ScanDefinition::new(
            context,
            "fn scan(a: u32, b: u32) -> u32 { return a + b; }".to_string(),
        )?;

        let stream_wgsl = wgsl::stream();

        let stream_def = StreamDefinition::new(context, stream_wgsl)?;

        let sync_indirect_wgsl = shared_wgsl::indirect_source(EmissionKind::TriangleVertices);
        let sync_indirect_def = MapDefinition::new(sync_indirect_wgsl)?;

        let deinterleave_pipeline = shared_wgsl::deinterleave(context)?;

        Ok(Self {
            count_def,
            scan_def,
            stream_def,
            sync_indirect_def,
            tri_table_buffer,
            edge_table_buffer,
            tri_count_table_buffer,
            deinterleave_pipeline,
        })
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "surface extraction passes grid, threshold, output bounds and world transform to the GPU"
)]
impl MarchingCubes {
    /// Emit a prefix of complete triangles within the vertex budget.
    /// The indirect draw count reports only written vertices. The grid must
    /// match the distance-field texture; zero and overflowing layouts fail
    /// before GPU work.
    pub fn execute(
        context: &WgpuContext,
        definition: &MarchingCubesDefinition,
        sdf: &GpuResource,
        grid: ExtractionGrid,
        threshold: f32,
        max_vertices: MeshVertexCount,
        scale: (f32, f32, f32),
        offset: (f32, f32, f32),
    ) -> Result<TriangleExtraction> {
        grid.ensure_sdf(sdf)?;
        grid.admit_triangle_prefix()?;
        let output_layout = VertexOutputLayout::new(max_vertices)?;

        // 1. Count Pass
        let counts_buffer = Buffer::new(
            context,
            grid.word_bytes(),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("counts_buffer").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination),
        )?;
        let counts_resource = GpuResource::Buffer(counts_buffer.clone());

        let mut count_params = PassParameters::new();
        grid.insert(&mut count_params);
        count_params.insert("max_vertices", max_vertices);
        count_params.insert("threshold", threshold);
        count_params.insert("scale_x", scale.0);
        count_params.insert("scale_y", scale.1);
        count_params.insert("scale_z", scale.2);
        count_params.insert("offset_x", offset.0);
        count_params.insert("offset_y", offset.1);
        count_params.insert("offset_z", offset.2);
        count_params.insert("tri_count_table", definition.tri_count_table_buffer.clone());

        Gather::execute_with_parameters(
            context,
            &definition.count_def,
            sdf,
            &counts_resource,
            Some(count_params),
        )?;

        // 2. Scan Pass
        let inclusive_offsets = Scan::execute(context, &definition.scan_def, &counts_buffer)?;

        // 3. Sync Indirect Pass (Map)
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
            EmissionKind::TriangleVertices.indirect_bytes(),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("marching_cubes_indirect").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::Indirect),
        )?;

        let mut sync_params = PassParameters::new();
        sync_params.insert("inclusive_offsets", inclusive_offsets.clone());
        sync_params.insert("indirect", output_indirect.clone());
        sync_params.insert("capacity", max_vertices);

        crate::Map::execute_with_parameters(
            context,
            &definition.sync_indirect_def,
            Some(&GpuResource::Buffer(dummy_in)),
            &GpuResource::Buffer(dummy_out),
            Some(sync_params),
        )?;

        // 4. Generate Pass (Stream)
        let mut gen_params = PassParameters::new();
        grid.insert(&mut gen_params);
        gen_params.insert("max_vertices", max_vertices);
        gen_params.insert("threshold", threshold);
        gen_params.insert("scale_x", scale.0);
        gen_params.insert("scale_y", scale.1);
        gen_params.insert("scale_z", scale.2);
        gen_params.insert("offset_x", offset.0);
        gen_params.insert("offset_y", offset.1);
        gen_params.insert("offset_z", offset.2);
        gen_params.insert("triTable", definition.tri_table_buffer.clone());
        gen_params.insert("edgeTable", definition.edge_table_buffer.clone());

        let output_vertices = Buffer::new(
            context,
            output_layout.storage_bytes(definition.deinterleave_pipeline.storage_stride()?),
            fabelgeist_gpu::data::BufferDefinition::storage()
                .with_label(("marching_cubes_output_vertices").into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination),
        )?;

        crate::Stream::execute(
            context,
            &definition.stream_def,
            sdf,
            &counts_buffer,
            &inclusive_offsets,
            &output_vertices,
            Some(gen_params),
        )?;

        let attributes = output_layout.project(
            context,
            &definition.deinterleave_pipeline,
            output_vertices,
            AttributeLabels {
                positions: "marching_cubes_positions".into(),
                normals: "marching_cubes_normals".into(),
            },
        )?;

        Ok(TriangleExtraction::new(attributes, output_indirect))
    }
}
