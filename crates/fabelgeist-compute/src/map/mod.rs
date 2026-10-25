use crate::prelude::*;
use fabelgeist_gpu::data::gpu::{ComputeEntryPoint::Named, ResourceDescriptor};
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use std::collections::HashMap;
use std::sync::Arc;

use bindings::MapBindings;
use recipe::MapRecipe;
pub use signature::MapSignature;

mod bindings;
mod program;
mod recipe;
mod signature;

type MapPipelineCache = ComputeSourceCache<MapRecipe, Arc<ResourcePipeline>>;

#[derive(Clone, Debug, Default)]
pub struct MapDefinition {
    pub code: String,
    cache: MapPipelineCache,
}

pub struct Map;

impl PartialEq for MapDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl MapDefinition {
    pub fn new(code: impl ToString) -> Result<Self> {
        let code = code.to_string();
        let _ = MapSignature::parse(&code)?;
        Ok(Self {
            code,
            cache: ComputeSourceCache::default(),
        })
    }

    pub fn build_pipeline(
        &self,
        context: &WgpuContext,
        input_res: Option<ResourceDescriptor>,
        output_res: ResourceDescriptor,
        secondary_resources: &HashMap<String, ResourceDescriptor>,
    ) -> Result<ResourcePipeline> {
        let signature = MapSignature::parse(&self.code)?;
        let bindings = MapBindings::new(&signature, input_res.as_ref(), secondary_resources);
        let mut full_code = String::new();
        bindings.declarations(&mut full_code, input_res.as_ref(), &output_res, secondary_resources)?;
        full_code.push('\n');
        // Preserve the authored source; validate the generated program below.
        full_code.push_str(&self.code);
        full_code.push('\n');
        match output_res {
            ResourceDescriptor::Buffer(_) => {
                full_code.push_str("@compute @workgroup_size(64, 1, 1)\n")
            }
            ResourceDescriptor::Texture2d(_) => {
                full_code.push_str("@compute @workgroup_size(16, 16, 1)\n")
            }
            ResourceDescriptor::Texture3d(_) => {
                full_code.push_str("@compute @workgroup_size(8, 8, 4)\n")
            }
        }
        full_code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
        full_code.push_str(&output_res.generate_prologue());
        let mut addressing = program::index(&mut full_code, &signature, &output_res)?;
        addressing = addressing.combine(program::size(&mut full_code, &signature, &output_res));
        addressing = addressing.combine(bindings.fetches(
            &mut full_code,
            input_res.as_ref(),
            &output_res,
            secondary_resources,
        )?);
        bindings.unpack_uniforms(&mut full_code);
        let map_call = format!("map({})", signature.call_arguments(bindings.primary()).join(", "));
        program::store(&mut full_code, &signature, &output_res, &map_call)?;
        full_code.push_str("}\n");
        let shader = ComputeShader::new(context, full_code)?;
        let pipeline = ComputePipeline::new(context, shader, Named("main"))?;
        ResourcePipeline::new(pipeline, DispatchResource::Output, addressing)
    }

    pub fn get_or_create_pipeline(
        &self,
        context: &WgpuContext,
        input_res: Option<ResourceDescriptor>,
        output_res: ResourceDescriptor,
        secondary_resources: &HashMap<String, ResourceDescriptor>,
    ) -> Result<Arc<ResourcePipeline>> {
        let key = MapRecipe {
            input: input_res.clone(),
            output: output_res.clone(),
            secondary: secondary_resources.into(),
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

impl Map {
    pub fn execute(
        context: &WgpuContext,
        definition: &MapDefinition,
        input: Option<&GpuResource>,
        output: &GpuResource,
    ) -> Result<()> {
        Self::execute_with_parameters(context, definition, input, output, None)
    }

    pub fn execute_with_parameters(
        context: &WgpuContext,
        definition: &MapDefinition,
        input: Option<&GpuResource>,
        output: &GpuResource,
        extra_parameters: Option<fabelgeist_gpu::data::gpu::parameters::PassParameters>,
    ) -> Result<()> {
        let sig = MapSignature::parse(&definition.code)?;

        let input_descriptor = input.and_then(|input| {
            sig.user_parameters().next().map(|parameter| {
                ResourceDescriptor::from_resource(input, parameter.value_type().clone())
            })
        });
        let output_descriptor =
            ResourceDescriptor::from_resource(output, sig.output_element_type().clone());

        let mut secondary_resources = HashMap::new();
        if let Some(extra) = extra_parameters.as_ref() {
            for (name, val) in &extra.parameters {
                match val {
                    fabelgeist_gpu::data::gpu::parameters::PassParameter::Buffer(_b) => {
                        if let Some(parameter) = sig.user_parameters().find(|parameter| parameter.name() == name)
                        {
                            secondary_resources
                                .insert(name.clone(), ResourceDescriptor::Buffer(parameter.value_type().clone()));
                        }
                    }
                    fabelgeist_gpu::data::gpu::parameters::PassParameter::Texture2d(t) => {
                        secondary_resources
                            .insert(name.clone(), ResourceDescriptor::Texture2d(t.format()));
                    }
                    fabelgeist_gpu::data::gpu::parameters::PassParameter::Texture3d(t) => {
                        secondary_resources
                            .insert(name.clone(), ResourceDescriptor::Texture3d(t.format()));
                    }
                    _ => {}
                }
            }
        }

        let pipeline_info = definition.get_or_create_pipeline(
            context,
            input_descriptor,
            output_descriptor,
            &secondary_resources,
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

        // Gather input info if needed
        if let Some(input) = input {
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
