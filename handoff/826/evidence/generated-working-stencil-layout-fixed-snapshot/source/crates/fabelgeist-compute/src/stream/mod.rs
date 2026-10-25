use crate::prelude::*;
use fabelgeist_gpu::data::gpu::ResourceDescriptor;
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use std::sync::Arc;

pub use signature::StreamSignature;

mod call;
mod program;
mod signature;

#[derive(Clone, Debug, Default)]
pub struct StreamDefinition {
    pub code: String,
    cache: ComputeSourceCache<ResourceDescriptor, Arc<ResourcePipeline>>,
}

pub struct Stream;

impl PartialEq for StreamDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl StreamDefinition {
    pub fn new(code: String) -> Result<Self> {
        let _ = StreamSignature::parse(&code)?;
        Ok(Self {
            code,
            cache: ComputeSourceCache::default(),
        })
    }

    pub fn build_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceDescriptor,
    ) -> Result<ResourcePipeline> {
        let signature = StreamSignature::parse(&self.code)?;
        program::build(context, &self.code, &signature, input_res)
    }

    pub fn get_or_create_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceDescriptor,
    ) -> Result<Arc<ResourcePipeline>> {
        self.cache
            .get_or_try_insert(context, &self.code, input_res.clone(), || {
                let pipeline = self.build_pipeline(context, input_res.clone())?;
                let arc_info = Arc::new(pipeline);
                Ok(arc_info)
            })
    }
}

impl Stream {
    /// Record work over the offset prefix for buffers, or input texture axes.
    /// Counts and inclusive offsets must describe that same input population;
    /// the authored writer must keep its writes inside the output allocation.
    pub fn execute(
        context: &WgpuContext,
        definition: &StreamDefinition,
        input: &GpuResource,
        counts: &fabelgeist_gpu::data::gpu::Buffer,
        inclusive_offsets: &fabelgeist_gpu::data::gpu::Buffer,
        output: &fabelgeist_gpu::data::gpu::Buffer,
        extra_parameters: Option<fabelgeist_gpu::data::gpu::parameters::PassParameters>,
    ) -> Result<()> {
        let sig = StreamSignature::parse(&definition.code)?;
        let pipeline = definition.get_or_create_pipeline(
            context,
            ResourceDescriptor::from_resource(input, sig.input_element_type().clone()),
        )?;

        let mut parameters = extra_parameters.unwrap_or_default();

        match input {
            GpuResource::Buffer(b) => parameters.insert("input", b.clone()),
            GpuResource::Texture2d(t) => parameters.insert("input", t.clone()),
            GpuResource::Texture3d(t) => parameters.insert("input", t.clone()),
        }

        parameters.insert("counts", counts.clone());
        parameters.insert("inclusive_offsets", inclusive_offsets.clone());
        parameters.insert("output", output.clone());

        let workgroups = match input {
            GpuResource::Buffer(_) => {
                // The offset table, rather than input T, defines this prefix.
                let offsets = GpuResource::Buffer(inclusive_offsets.clone());
                pipeline.workgroups(&offsets)?
            }
            _ => pipeline.workgroups(input)?,
        };

        fabelgeist_gpu::data::gpu::ComputePass::dispatch(
            context,
            pipeline.pipeline().clone(),
            parameters,
            workgroups,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
