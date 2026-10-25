use crate::prelude::*;
use crate::source_error::{FluidOperation, GeneratedBinding};
use fabelgeist_gpu::data::gpu::ComputeEntryPoint::Named;
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use std::sync::Arc;

mod program;

type AdvectPipelineCache = ComputeSourceCache<AdvectRecipe, Arc<ResourcePipeline>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct AdvectRecipe {
    velocity: ResourceDescriptor,
    quantity: ResourceDescriptor,
    output: ResourceDescriptor,
    boundary_mode: BoundaryMode,
}

#[derive(Clone, Debug, Default)]
pub struct AdvectDefinition {
    pub mode: BoundaryMode,
    cache: AdvectPipelineCache,
}

pub struct Advect;

impl AdvectDefinition {
    pub fn new(mode: BoundaryMode) -> Self {
        Self {
            mode,
            cache: ComputeSourceCache::default(),
        }
    }
}

impl AdvectDefinition {
    pub fn build_pipeline(
        &self,
        context: &WgpuContext,
        velocity_res: ResourceDescriptor,
        quantity_res: ResourceDescriptor,
        output_res: ResourceDescriptor,
    ) -> Result<ResourcePipeline> {
        let full_code = program::source(self.mode, &velocity_res, &quantity_res, &output_res)?;
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
        velocity_res: ResourceDescriptor,
        quantity_res: ResourceDescriptor,
        output_res: ResourceDescriptor,
    ) -> Result<Arc<ResourcePipeline>> {
        let key = AdvectRecipe {
            velocity: velocity_res.clone(),
            quantity: quantity_res.clone(),
            output: output_res.clone(),
            boundary_mode: self.mode,
        };
        self.cache.get_or_try_insert(context, "", key, || {
            let pipeline = self.build_pipeline(context, velocity_res, quantity_res, output_res)?;
            let arc_p = Arc::new(pipeline);
            Ok(arc_p)
        })
    }
}

impl Advect {
    /// Advect a quantity using velocity measured in texels per time unit.
    /// `dt` uses that same time unit; the native kernel divides velocity by
    /// its texture dimensions before applying the step in normalized coordinates.
    pub fn execute(
        context: &WgpuContext,
        definition: &AdvectDefinition,
        velocity: &GpuResource,
        quantity: &GpuResource,
        dt: f32,
        output: &GpuResource,
    ) -> Result<()> {
        let velocity_descriptor = ResourceDescriptor::from_resource(
            velocity,
            ResourceBaseType::Vec2(Box::new(ResourceBaseType::F32)),
        );
        let quantity_descriptor =
            GeneratedBinding::Quantity.texture_descriptor(FluidOperation::Advect, quantity)?;
        let output_descriptor =
            GeneratedBinding::Output.texture_descriptor(FluidOperation::Advect, output)?;

        let pipeline = definition.get_or_create_pipeline(
            context,
            velocity_descriptor,
            quantity_descriptor,
            output_descriptor,
        )?;

        let mut parameters = fabelgeist_gpu::data::gpu::parameters::PassParameters::new();
        GeneratedBinding::Velocity.insert_texture(
            FluidOperation::Advect,
            &mut parameters,
            velocity,
        )?;
        GeneratedBinding::Quantity.insert_texture(
            FluidOperation::Advect,
            &mut parameters,
            quantity,
        )?;
        GeneratedBinding::Output.insert_texture(FluidOperation::Advect, &mut parameters, output)?;
        let size = GeneratedBinding::Output.texture_size(FluidOperation::Advect, output)?;
        parameters.insert("size", size);
        parameters.insert("dt", dt);

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
