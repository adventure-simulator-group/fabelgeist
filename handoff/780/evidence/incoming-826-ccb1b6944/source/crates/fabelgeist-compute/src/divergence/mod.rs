use crate::prelude::*;
use crate::source_error::{FluidOperation, GeneratedBinding};
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use fabelgeist_gpu::data::gpu::{ComputeEntryPoint::Named, ResourceDescriptor};
use std::sync::Arc;

mod program;

type DivergencePipelineCache = ComputeSourceCache<DivergenceRecipe, Arc<ResourcePipeline>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DivergenceRecipe {
    input: ResourceDescriptor,
    output: ResourceDescriptor,
    boundary_mode: BoundaryMode,
}

#[derive(Clone, Debug, Default)]
pub struct DivergenceDefinition {
    pub boundary_mode: BoundaryMode,
    cache: DivergencePipelineCache,
}

pub struct Divergence;

impl DivergenceDefinition {
    pub fn new(boundary_mode: BoundaryMode) -> Self {
        Self {
            boundary_mode,
            cache: ComputeSourceCache::default(),
        }
    }
}

impl DivergenceDefinition {
    pub fn build_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceDescriptor,
        output_res: ResourceDescriptor,
    ) -> Result<ResourcePipeline> {
        let full_code = program::source(self.boundary_mode, &input_res, &output_res)?;
        let shader = ComputeShader::new(context, full_code)?;
        let pipeline = ComputePipeline::new(context, shader, Named("main"))?;
        ResourcePipeline::new(
            pipeline,
            DispatchResource::Output,
            TextureAddressing::Coordinates,
        )
    }

    pub fn get_or_create_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceDescriptor,
        output_res: ResourceDescriptor,
    ) -> Result<Arc<ResourcePipeline>> {
        let key = DivergenceRecipe {
            input: input_res.clone(),
            output: output_res.clone(),
            boundary_mode: self.boundary_mode,
        };
        self.cache.get_or_try_insert(context, "", key, || {
            let pipeline = self.build_pipeline(context, input_res, output_res)?;
            let arc_p = Arc::new(pipeline);
            Ok(arc_p)
        })
    }
}

impl Divergence {
    /// Apply the centered velocity difference between adjacent texels.
    /// `half_inverse_cell_size` is the numerical coefficient `1 / (2 * dx)`
    /// in the caller's shared velocity and grid-spacing frame.
    pub fn execute(
        context: &WgpuContext,
        definition: &DivergenceDefinition,
        velocity: &GpuResource,
        half_inverse_cell_size: f32,
        output: &GpuResource,
    ) -> Result<()> {
        let input_descriptor = ResourceDescriptor::from_resource(
            velocity,
            ResourceBaseType::Vec2(Box::new(ResourceBaseType::F32)),
        );
        let output_descriptor = ResourceDescriptor::from_resource(output, ResourceBaseType::F32);

        let pipeline =
            definition.get_or_create_pipeline(context, input_descriptor, output_descriptor)?;

        let mut parameters = fabelgeist_gpu::data::gpu::parameters::PassParameters::new();
        GeneratedBinding::Velocity.insert_texture(
            FluidOperation::Divergence,
            &mut parameters,
            velocity,
        )?;
        GeneratedBinding::Output.insert_texture(
            FluidOperation::Divergence,
            &mut parameters,
            output,
        )?;
        parameters.insert("half_inverse_cell_size", half_inverse_cell_size);

        let workgroups = pipeline.workgroups(output)?;

        fabelgeist_gpu::data::gpu::ComputePass::dispatch(
            context,
            pipeline.pipeline().clone(),
            parameters,
            workgroups,
        )?;

        Ok(())
    }
}
