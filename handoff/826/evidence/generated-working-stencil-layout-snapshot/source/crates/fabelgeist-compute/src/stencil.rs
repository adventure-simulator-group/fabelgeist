//! Authored resource and neighborhood Stencil programs.

use crate::prelude::*;
use fabelgeist_gpu::data::gpu::ResourceDescriptor;
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use std::collections::HashMap;
use std::sync::Arc;

pub use neighborhood::NeighborhoodDimension;
pub use neighborhood::layout::{
    NeighborhoodAxis, NeighborhoodLayoutError, NeighborhoodLayoutResult,
};
use recipe::StencilRecipe;
pub use signature::StencilSignature;

mod bindings;
mod neighborhood;
mod program;
mod recipe;
mod signature;

type StencilPipelineCache = ComputeSourceCache<StencilRecipe, Arc<ResourcePipeline>>;

#[derive(Clone, Debug, Default)]
pub struct StencilDefinition {
    pub code: String,
    pub boundary_mode: BoundaryMode,
    cache: StencilPipelineCache,
}

pub struct Stencil;

impl PartialEq for StencilDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.boundary_mode == other.boundary_mode
    }
}

impl StencilDefinition {
    pub fn new(code: String) -> Result<Self> {
        let _ = StencilSignature::parse(&code)?;
        Ok(Self {
            code,
            ..Default::default()
        })
    }

    pub fn new_with_boundary(code: String, boundary_mode: BoundaryMode) -> Result<Self> {
        let _ = StencilSignature::parse(&code)?;
        Ok(Self {
            code,
            boundary_mode,
            cache: ComputeSourceCache::default(),
        })
    }

    pub fn build_pipeline(
        &self,
        context: &WgpuContext,
        input: ResourceDescriptor,
        output: ResourceDescriptor,
        secondary: &HashMap<String, ResourceDescriptor>,
    ) -> Result<ResourcePipeline> {
        program::build(context, self, input, output, secondary)
    }

    pub fn get_or_create_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceDescriptor,
        output_res: ResourceDescriptor,
        secondary_resources: &HashMap<String, ResourceDescriptor>,
    ) -> Result<Arc<ResourcePipeline>> {
        let key = StencilRecipe {
            input: input_res.clone(),
            output: output_res.clone(),
            secondary: secondary_resources.into(),
            boundary_mode: self.boundary_mode,
        };

        self.cache.get_or_try_insert(context, &self.code, key, || {
            let pipeline_info = self.build_pipeline(
                context,
                input_res.clone(),
                output_res.clone(),
                secondary_resources,
            )?;
            let arc_info = Arc::new(pipeline_info);
            Ok(arc_info)
        })
    }
}

impl Stencil {
    pub fn execute(
        context: &WgpuContext,
        definition: &StencilDefinition,
        input: &GpuResource,
        output: &GpuResource,
    ) -> Result<()> {
        Self::execute_with_parameters(context, definition, input, output, None)
    }

    pub fn execute_with_parameters(
        context: &WgpuContext,
        definition: &StencilDefinition,
        input: &GpuResource,
        output: &GpuResource,
        extra_parameters: Option<fabelgeist_gpu::data::gpu::parameters::PassParameters>,
    ) -> Result<()> {
        let sig = StencilSignature::parse(&definition.code)?;

        let input_descriptor =
            ResourceDescriptor::from_resource(input, sig.input_element_type().clone());
        let output_descriptor =
            ResourceDescriptor::from_resource(output, sig.output_element_type().clone());

        let mut parameters = extra_parameters.unwrap_or_default();

        let secondary_resources = bindings::secondary_resources(&sig, &parameters);

        let pipeline_info = definition.get_or_create_pipeline(
            context,
            input_descriptor,
            output_descriptor,
            &secondary_resources,
        )?;
        let pipeline = pipeline_info.pipeline();

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
        if let Some(neighborhood) = sig.neighborhood() {
            neighborhood.admit_resources(
                &sig,
                definition.boundary_mode,
                &pipeline_info,
                input,
                output,
            )?;
        }

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
