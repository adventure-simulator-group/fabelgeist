use crate::extraction::EmissionKind;
use crate::extraction::wgsl as shared_wgsl;
use crate::{GatherDefinition, MapDefinition, ScanDefinition, StreamDefinition};
use anyhow::Result;
use fabelgeist_gpu::globals::WgpuContext;

pub use execution::DualContouring;

mod execution;
mod wgsl;

#[derive(Clone, Debug)]
pub struct DualContouringDefinition {
    vertex_count_def: GatherDefinition,
    index_count_def: GatherDefinition,
    scan_def: ScanDefinition,
    vertex_stream_def: StreamDefinition,
    index_stream_def: StreamDefinition,
    sync_indirect_def: MapDefinition,
    deinterleave_pipeline: crate::ResourcePipeline,
}

impl PartialEq for DualContouringDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_count_def == other.vertex_count_def
            && self.index_count_def == other.index_count_def
            && self.scan_def == other.scan_def
            && self.vertex_stream_def == other.vertex_stream_def
            && self.index_stream_def == other.index_stream_def
            && self.sync_indirect_def == other.sync_indirect_def
    }
}

impl DualContouringDefinition {
    pub fn new(context: &WgpuContext) -> Result<Self> {

        let vertex_count_wgsl = wgsl::vertex_count();

        let index_count_wgsl = wgsl::index_count();

        let vertex_count_def = GatherDefinition::new(context, vertex_count_wgsl)?;
        let index_count_def = GatherDefinition::new(context, index_count_wgsl)?;
        let scan_def = ScanDefinition::new(
            context,
            "fn scan(a: u32, b: u32) -> u32 { return a + b; }".to_string(),
        )?;

        let vertex_stream_wgsl = wgsl::vertex_stream();

        let vertex_stream_def = StreamDefinition::new(context, vertex_stream_wgsl)?;

        let index_stream_wgsl = wgsl::index_stream();

        let index_stream_def = StreamDefinition::new(context, index_stream_wgsl)?;

        let sync_indirect_wgsl = shared_wgsl::indirect_source(EmissionKind::QuadIndices);
        let sync_indirect_def = MapDefinition::new(sync_indirect_wgsl)?;

        let deinterleave_pipeline = shared_wgsl::deinterleave(context)?;

        Ok(Self {
            vertex_count_def,
            index_count_def,
            scan_def,
            vertex_stream_def,
            index_stream_def,
            sync_indirect_def,
            deinterleave_pipeline,
        })
    }
}
