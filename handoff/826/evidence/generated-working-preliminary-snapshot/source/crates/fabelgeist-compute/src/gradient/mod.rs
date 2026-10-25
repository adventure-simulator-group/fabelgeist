use crate::prelude::*;
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use fabelgeist_gpu::data::gpu::{ComputeEntryPoint::Named, ResourceDescriptor};
use std::sync::Arc;

mod program;

type GradientPipelineCache = ComputeSourceCache<GradientRecipe, Arc<ResourcePipeline>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct GradientRecipe {
    velocity: ResourceDescriptor,
    pressure: ResourceDescriptor,
    output: ResourceDescriptor,
    boundary_mode: BoundaryMode,
}

#[derive(Clone, Debug, Default)]
pub struct GradientDefinition {
    pub boundary_mode: BoundaryMode,
    cache: GradientPipelineCache,
}

pub struct Gradient;

impl GradientDefinition {
    pub fn new(boundary_mode: BoundaryMode) -> Self {
        Self {
            boundary_mode,
            cache: ComputeSourceCache::default(),
        }
    }
}

impl GradientDefinition {
    pub fn build_pipeline(
        &self,
        context: &WgpuContext,
        velocity_res: ResourceDescriptor,
        pressure_res: ResourceDescriptor,
        output_res: ResourceDescriptor,
    ) -> Result<ResourcePipeline> {
        let full_code = program::source(
            self.boundary_mode,
            &velocity_res,
            &pressure_res,
            &output_res,
        )?;
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
        pressure_res: ResourceDescriptor,
        output_res: ResourceDescriptor,
    ) -> Result<Arc<ResourcePipeline>> {
        let key = GradientRecipe {
            velocity: velocity_res.clone(),
            pressure: pressure_res.clone(),
            output: output_res.clone(),
            boundary_mode: self.boundary_mode,
        };
        self.cache.get_or_try_insert(context, "", key, || {
            let pipeline = self.build_pipeline(context, velocity_res, pressure_res, output_res)?;
            let arc_p = Arc::new(pipeline);
            Ok(arc_p)
        })
    }
}

impl Gradient {
    /// Subtract the centered pressure difference from velocity.
    /// `half_inverse_cell_size` is the numerical coefficient `1 / (2 * dx)`
    /// in the caller's shared pressure, velocity and grid-spacing frame.
    pub fn execute(
        context: &WgpuContext,
        definition: &GradientDefinition,
        velocity: &GpuResource,
        pressure: &GpuResource,
        half_inverse_cell_size: f32,
        output: &GpuResource,
    ) -> Result<()> {
        let velocity_descriptor = ResourceDescriptor::from_resource(
            velocity,
            ResourceBaseType::Vec2(Box::new(ResourceBaseType::F32)),
        );
        let pressure_descriptor =
            ResourceDescriptor::from_resource(pressure, ResourceBaseType::F32);
        let output_descriptor = ResourceDescriptor::from_resource(
            output,
            ResourceBaseType::Vec2(Box::new(ResourceBaseType::F32)),
        );

        let pipeline = definition.get_or_create_pipeline(
            context,
            velocity_descriptor,
            pressure_descriptor,
            output_descriptor,
        )?;

        let mut parameters = fabelgeist_gpu::data::gpu::parameters::PassParameters::new();
        parameters.insert(
            "velocity",
            match velocity {
                GpuResource::Texture2d(t) => PassParameter::Texture2d(t.clone()),
                GpuResource::Texture3d(t) => PassParameter::Texture3d(t.clone()),
                _ => return Err(anyhow!("Velocity must be a texture")),
            },
        );
        parameters.insert(
            "pressure",
            match pressure {
                GpuResource::Texture2d(t) => PassParameter::Texture2d(t.clone()),
                GpuResource::Texture3d(t) => PassParameter::Texture3d(t.clone()),
                _ => return Err(anyhow!("Pressure must be a texture")),
            },
        );
        parameters.insert(
            "output",
            match output {
                GpuResource::Texture2d(t) => PassParameter::Texture2d(t.clone()),
                GpuResource::Texture3d(t) => PassParameter::Texture3d(t.clone()),
                _ => return Err(anyhow!("Output must be a texture")),
            },
        );

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
