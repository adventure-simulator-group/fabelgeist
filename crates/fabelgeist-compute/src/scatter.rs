use crate::prelude::*;
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use std::sync::Arc;

pub use signature::ScatterSignature;

mod program;
mod signature;

type ScatterPipelineCache = ComputeSourceCache<ScatterRecipe, Arc<ResourcePipeline>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ScatterRecipe {
    input: ResourceDescriptor,
    output: ResourceDescriptor,
}

#[derive(Clone, Debug, Default)]
pub struct ScatterDefinition {
    pub code: String,
    cache: ScatterPipelineCache,
}

pub struct Scatter;

impl PartialEq for ScatterDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl ScatterDefinition {
    pub fn new(code: String) -> Result<Self> {
        let _ = ScatterSignature::parse(&code)?;
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
        let signature = ScatterSignature::parse(&self.code)?;
        program::build(context, &self.code, &signature, input_res, output_res)
    }

    pub fn get_or_create_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceDescriptor,
        output_res: ResourceDescriptor,
    ) -> Result<Arc<ResourcePipeline>> {
        let recipe = ScatterRecipe {
            input: input_res.clone(),
            output: output_res.clone(),
        };
        self.cache
            .get_or_try_insert(context, &self.code, recipe, || {
                let pipeline_info = self.build_pipeline(context, input_res, output_res)?;
                let arc_info = Arc::new(pipeline_info);
                Ok(arc_info)
            })
    }
}

impl Scatter {
    pub fn execute(
        context: &WgpuContext,
        definition: &ScatterDefinition,
        input: &GpuResource,
        output: &GpuResource,
    ) -> Result<()> {
        let signature = ScatterSignature::parse(&definition.code)?;
        let pipeline_info = definition.get_or_create_pipeline(
            context,
            ResourceDescriptor::from_resource(input, signature.input_element_type().clone()),
            ResourceDescriptor::from_resource(output, signature.output_element_type().clone()),
        )?;
        let pipeline = pipeline_info.pipeline();

        let mut parameters = fabelgeist_gpu::data::gpu::parameters::PassParameters::new();
        // Input resource determines grid size
        match input {
            GpuResource::Buffer(buffer) => {
                parameters.insert("input", buffer.clone());
            }
            GpuResource::Texture2d(texture) => {
                parameters.insert("input", texture.clone());
            }
            GpuResource::Texture3d(texture) => {
                parameters.insert("input", texture.clone());
            }
        };

        // Scatter output info
        match output {
            GpuResource::Buffer(b) => {
                parameters.insert("output", b.clone());
            }
            GpuResource::Texture2d(t) => {
                parameters.insert("output", t.clone());
            }
            GpuResource::Texture3d(t) => {
                parameters.insert("output", t.clone());
            }
        }

        let workgroups = pipeline_info.workgroups(input)?;

        fabelgeist_gpu::data::gpu::ComputePass::dispatch(
            context,
            pipeline.clone(),
            parameters,
            workgroups,
        )?;

        Ok(())
    }
}
