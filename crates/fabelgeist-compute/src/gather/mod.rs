use crate::prelude::*;
use fabelgeist_gpu::data::gpu::ResourceDescriptor;
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use std::sync::Arc;

use recipe::GatherRecipe;
pub use signature::GatherSignature;

mod program;
mod recipe;
mod signature;

type GatherPipelineCache = ComputeSourceCache<GatherRecipe, Arc<ResourcePipeline>>;

#[derive(Clone, Debug, Default)]
pub struct GatherDefinition {
    pub code: String,
    cache: GatherPipelineCache,
}

pub struct Gather;

impl PartialEq for GatherDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl GatherDefinition {
    pub fn new(code: String) -> Result<Self> {
        let _ = GatherSignature::parse(&code)?;
        Ok(Self {
            code,
            cache: ComputeSourceCache::default(),
        })
    }

    pub fn build_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceDescriptor,
        output_res: ResourceDescriptor,
    ) -> Result<ResourcePipeline> {
        let signature = GatherSignature::parse(&self.code)?;
        program::build(context, &self.code, &signature, input_res, output_res)
    }

    pub fn get_or_create_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceDescriptor,
        output_res: ResourceDescriptor,
    ) -> Result<Arc<ResourcePipeline>> {
        self.cache.get_or_try_insert(
            context,
            &self.code,
            GatherRecipe {
                input: input_res.clone(),
                output: output_res.clone(),
            },
            || {
                let pipeline_info =
                    self.build_pipeline(context, input_res.clone(), output_res.clone())?;
                let arc_info = Arc::new(pipeline_info);
                Ok(arc_info)
            },
        )
    }
}

impl Gather {
    pub fn execute(
        context: &WgpuContext,
        definition: &GatherDefinition,
        input: &GpuResource,
        output: &GpuResource,
    ) -> Result<()> {
        Self::execute_with_parameters(context, definition, input, output, None)
    }

    pub fn execute_with_parameters(
        context: &WgpuContext,
        definition: &GatherDefinition,
        input: &GpuResource,
        output: &GpuResource,
        extra_parameters: Option<fabelgeist_gpu::data::gpu::parameters::PassParameters>,
    ) -> Result<()> {
        let sig = GatherSignature::parse(&definition.code)?;
        let pipeline_info = definition.get_or_create_pipeline(
            context,
            ResourceDescriptor::from_resource(input, sig.input_element_type().clone()),
            ResourceDescriptor::from_resource(output, sig.output_element_type().clone()),
        )?;
        let pipeline = pipeline_info.pipeline();

        let mut parameters = extra_parameters.unwrap_or_default();

        // Output resource determines grid size
        match output {
            GpuResource::Buffer(buffer) => {
                parameters.insert("output", buffer.clone());
            }
            GpuResource::Texture2d(texture) => {
                parameters.insert("output", texture.clone());
            }
            GpuResource::Texture3d(texture) => {
                parameters.insert("output", texture.clone());
            }
        };

        // Gather input info
        match input {
            GpuResource::Buffer(b) => {
                parameters.insert("input", b.clone());
            }
            GpuResource::Texture2d(t) => {
                parameters.insert("input", t.clone());
            }
            GpuResource::Texture3d(t) => {
                parameters.insert("input", t.clone());
            }
        }

        let workgroups = pipeline_info.workgroups(output)?;

        fabelgeist_gpu::data::gpu::ComputePass::dispatch(
            context,
            pipeline.clone(),
            parameters,
            workgroups,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
