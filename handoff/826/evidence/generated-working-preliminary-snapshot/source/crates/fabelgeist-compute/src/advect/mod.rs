use crate::prelude::*;
use fabelgeist_gpu::data::gpu::ComputeEntryPoint::Named;
use fabelgeist_gpu::data::gpu::parameters::PassParameter;
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
        let quantity_descriptor = ResourceDescriptor::from_resource(
            quantity,
            match quantity {
                GpuResource::Texture2d(t) => ResourceBaseType::from_texture_format(t.format()),
                GpuResource::Texture3d(t) => ResourceBaseType::from_texture_format(t.format()),
                _ => return Err(anyhow!("Quantity must be a texture")),
            },
        );
        let output_descriptor = ResourceDescriptor::from_resource(
            output,
            match output {
                GpuResource::Texture2d(t) => ResourceBaseType::from_texture_format(t.format()),
                GpuResource::Texture3d(t) => ResourceBaseType::from_texture_format(t.format()),
                _ => return Err(anyhow!("Output must be a texture")),
            },
        );

        let pipeline = definition.get_or_create_pipeline(
            context,
            velocity_descriptor,
            quantity_descriptor,
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
            "quantity",
            match quantity {
                GpuResource::Texture2d(t) => PassParameter::Texture2d(t.clone()),
                GpuResource::Texture3d(t) => PassParameter::Texture3d(t.clone()),
                _ => return Err(anyhow!("Quantity must be a texture")),
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

        let size = match output {
            GpuResource::Texture2d(t) => fabelgeist_gpu::data::vector::Vec4::new(
                t.extent().width as f32,
                t.extent().height as f32,
                1.0,
                1.0,
            ),
            GpuResource::Texture3d(t) => fabelgeist_gpu::data::vector::Vec4::new(
                t.extent().width as f32,
                t.extent().height as f32,
                t.extent().depth_or_array_layers as f32,
                1.0,
            ),
            _ => return Err(anyhow!("Output must be a texture")),
        };

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
