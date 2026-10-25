use crate::prelude::*;
use fabelgeist_gpu::data::gpu::ResourceDescriptor;
use fabelgeist_gpu::data::gpu::signature::{CoordinateParameter, CoordinateShape};
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use std::collections::HashMap;
use std::sync::Arc;

use recipe::MapRecipe;

mod recipe;

type MapPipelineCache = ComputeSourceCache<MapRecipe, Arc<ResourcePipeline>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MapSignature {
    pub map_args: Vec<String>, // Raw arguments: e.g. "index: vec2<u32>", "in_val: f32"
    pub output_element_type: ResourceBaseType,
    pub index: Option<CoordinateParameter>,
    pub size: Option<CoordinateParameter>,
    pub user_params: Vec<(String, ResourceBaseType)>,
    pub param_names: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct MapDefinition {
    pub code: String,
    cache: MapPipelineCache,
}

impl Default for MapDefinition {
    fn default() -> Self {
        Self {
            code: String::new(),
            cache: ComputeSourceCache::default(),
        }
    }
}

impl PartialEq for MapDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl MapDefinition {
    pub fn new(code: impl ToString) -> Result<Self> {
        let code = code.to_string();
        let _ = Self::parse_signature(&code)?;
        Ok(Self {
            code,
            cache: ComputeSourceCache::default(),
        })
    }

    pub fn parse_signature(code: &str) -> Result<MapSignature> {
        let re = regex::Regex::new(r"(?s)fn\s+map\s*\(([^)]*)\)\s*(?:->\s*([^\{]+))?\{")?;
        let caps = re
            .captures(code)
            .ok_or_else(|| anyhow::anyhow!("Compute shader must define a 'map' function"))?;

        let args_str = &caps[1];
        let output_type_raw = caps.get(2).map(|m| m.as_str().trim().to_string());

        let mut args = Vec::new();
        let mut bracket_level = 0;
        let mut current_arg = String::new();
        for c in args_str.chars() {
            match c {
                '<' => {
                    bracket_level += 1;
                    current_arg.push(c);
                }
                '>' => {
                    bracket_level -= 1;
                    current_arg.push(c);
                }
                ',' if bracket_level == 0 => {
                    if !current_arg.trim().is_empty() {
                        args.push(current_arg.trim().to_string());
                    }
                    current_arg = String::new();
                }
                _ => current_arg.push(c),
            }
        }
        if !current_arg.trim().is_empty() {
            args.push(current_arg.trim().to_string());
        }

        let mut index: Option<CoordinateParameter> = None;
        let mut size: Option<CoordinateParameter> = None;
        let mut user_params = vec![];
        let mut param_names = vec![];

        for arg in &args {
            let parts: Vec<&str> = arg.split(':').map(|s| s.trim()).collect();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!("Invalid map parameter format: {}", arg));
            }
            let name = parts[0];
            let ty_str = parts[1];

            param_names.push(name.to_string());

            if name == "index" {
                index = Some(CoordinateParameter::from_native(name, ty_str)?);
            } else if name == "size" {
                size = Some(CoordinateParameter::from_native(name, ty_str)?);
            } else {
                user_params.push((
                    name.to_string(),
                    fabelgeist_gpu::data::gpu::signature::parse_base_type(ty_str),
                ));
            }
        }

        let output_element_type = if let Some(out_rt) = output_type_raw {
            fabelgeist_gpu::data::gpu::signature::parse_base_type(&out_rt)
        } else {
            return Err(anyhow::anyhow!("Map function must return a type."));
        };

        Ok(MapSignature {
            map_args: args,
            output_element_type,
            index,
            size,
            user_params,
            param_names,
        })
    }

    pub fn build_pipeline(
        &self,
        context: &WgpuContext,
        input_res: Option<ResourceDescriptor>,
        output_res: ResourceDescriptor,
        secondary_resources: &HashMap<String, ResourceDescriptor>,
    ) -> Result<ResourcePipeline> {
        let sig = Self::parse_signature(&self.code)?;
        let mut addressing = TextureAddressing::Coordinates;

        // 1. Build the full WGSL code
        let mut full_code = String::new();

        // 2. Determine input/extra parameter mapping
        let (input_param_name, input_element_type, extra_params) = if let Some(_res) = &input_res {
            if sig.user_params.is_empty() {
                (None, None, &sig.user_params[..])
            } else {
                (
                    Some(&sig.user_params[0].0),
                    Some(sig.user_params[0].1.clone()),
                    &sig.user_params[1..],
                )
            }
        } else {
            (None, None, &sig.user_params[..])
        };

        if let Some(res) = &input_res {
            full_code.push_str(&res.to_wgsl_input_binding(0, 0, "input"));
        }

        full_code.push_str(&output_res.to_wgsl_output_binding(0, 1, "output"));

        let mut uniform_params = Vec::new();
        let mut resource_params = Vec::new();
        for (name, ty) in extra_params {
            if secondary_resources.contains_key(name) {
                resource_params.push((name.clone(), ty.clone()));
            } else {
                uniform_params.push((name.clone(), ty.clone()));
            }
        }

        let mut current_binding = 2;
        for (name, _) in &resource_params {
            let res_desc = &secondary_resources[name];
            full_code.push_str(&res_desc.to_wgsl_input_binding(0, current_binding, name));
            current_binding += 1;
        }

        if !uniform_params.is_empty() {
            full_code.push_str("\nstruct Parameters {\n");
            for (name, ty) in &uniform_params {
                full_code.push_str(&format!("    {}: {},\n", name, ty.as_str()));
            }
            full_code.push_str("};\n");
            full_code.push_str(&format!(
                "@group(0) @binding({}) var<uniform> _params: Parameters;\n",
                current_binding
            ));
        }

        full_code.push('\n');
        // No modification to user's code needed; paste it directly because it contains no banned types
        full_code.push_str(&self.code);
        full_code.push('\n');

        // Main compute function
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

        // Generate the user's requested index type
        if let Some(parameter) = &sig.index {
            if parameter.shape() == CoordinateShape::Scalar {
                addressing = addressing.combine(TextureAddressing::LinearIndex);
                match output_res {
                    ResourceDescriptor::Buffer(_) => full_code.push_str("    let index = _global_index;\n"),
                    ResourceDescriptor::Texture2d(_) => full_code.push_str("    let index = _global_index.y * tex_dim.x + _global_index.x;\n"),
                    ResourceDescriptor::Texture3d(_) => full_code.push_str("    let index = (_global_index.z * tex_dim.y + _global_index.y) * tex_dim.x + _global_index.x;\n"),
                }
            } else if parameter.shape() == CoordinateShape::Plane {
                if !matches!(
                    output_res,
                    ResourceDescriptor::Texture2d(_) | ResourceDescriptor::Texture3d(_)
                ) {
                    return Err(anyhow::anyhow!("vec2 index only supported for Textures"));
                }
                full_code.push_str("    let index = _global_index.xy;\n");
            } else if parameter.shape() == CoordinateShape::Volume {
                if !matches!(output_res, ResourceDescriptor::Texture3d(_)) {
                    return Err(anyhow::anyhow!("vec3 index only supported for Texture3d"));
                }
                full_code.push_str("    let index = _global_index;\n");
            }
        }

        // Generate the user's requested size type
        if let Some(parameter) = &sig.size {
            match parameter.shape() {
                CoordinateShape::Scalar => {
                    addressing = TextureAddressing::LinearPopulation;
                    match output_res {
                        ResourceDescriptor::Buffer(_) => {
                            full_code.push_str("    let size = arrayLength(&output);\n")
                        }
                        ResourceDescriptor::Texture2d(_) => {
                            full_code.push_str("    let size = tex_dim.x * tex_dim.y;\n")
                        }
                        ResourceDescriptor::Texture3d(_) => {
                            full_code.push_str("    let size = tex_dim.x * tex_dim.y * tex_dim.z;\n")
                        }
                    }
                }
                CoordinateShape::Plane => match output_res {
                    ResourceDescriptor::Buffer(_) => {
                        full_code.push_str("    let size = vec2<u32>(arrayLength(&output), 1u);\n")
                    }
                    ResourceDescriptor::Texture2d(_) => {
                        full_code.push_str("    let size = tex_dim;\n")
                    }
                    ResourceDescriptor::Texture3d(_) => {
                        full_code.push_str("    let size = tex_dim.xy;\n")
                    }
                },
                CoordinateShape::Volume => match output_res {
                    ResourceDescriptor::Buffer(_) => full_code
                        .push_str("    let size = vec3<u32>(arrayLength(&output), 1u, 1u);\n"),
                    ResourceDescriptor::Texture2d(_) => {
                        full_code.push_str("    let size = vec3<u32>(tex_dim, 1u);\n")
                    }
                    ResourceDescriptor::Texture3d(_) => {
                        full_code.push_str("    let size = tex_dim;\n")
                    }
                },
            }
        }

        // Fetch input value if the map function uses it
        if let Some(res) = &input_res {
            let fetch = res.generate_fetch(
                "input",
                &output_res,
                input_element_type
                    .as_ref()
                    .unwrap_or(&ResourceBaseType::F32),
            );
            addressing = addressing.combine(fetch.addressing());
            full_code.push_str(fetch.code());
        }

        // Fetch resource params
        for (name, ty) in &resource_params {
            let res_desc = &secondary_resources[name];
            let fetch = res_desc.generate_fetch(name, &output_res, ty);
            addressing = addressing.combine(fetch.addressing());
            let fetch_code_custom = fetch
                .code()
                .replace("_in_global_index", &format!("_in_global_index_{}", name))
                .replace(
                    "_fetch_out_tex_dim",
                    &format!("_fetch_out_tex_dim_{}", name),
                )
                .replace("in_tex_dim", &format!("in_tex_dim_{}", name))
                .replace("let in_val = ", &format!("let {} = ", name));
            full_code.push_str(&fetch_code_custom);
        }

        // Prepare map call arguments and unpack extra params
        if !uniform_params.is_empty() {
            for (name, _) in &uniform_params {
                full_code.push_str(&format!("    let {} = _params.{};\n", name, name));
            }
        }

        let mut map_call_args = vec![];
        for name in &sig.param_names {
            if sig.index.as_ref().map(CoordinateParameter::name) == Some(name.as_str()) {
                map_call_args.push("index".to_string());
            } else if sig.size.as_ref().map(CoordinateParameter::name) == Some(name.as_str()) {
                map_call_args.push("size".to_string());
            } else if Some(name) == input_param_name {
                map_call_args.push("in_val".to_string());
            } else {
                map_call_args.push(name.to_string());
            }
        }
        let map_call = format!("map({})", map_call_args.join(", "));

        // Output logic
        match output_res {
            ResourceDescriptor::Buffer(_) => {
                full_code.push_str(&format!("    output[_global_index] = {};\n", map_call));
            }
            ResourceDescriptor::Texture2d(_) | ResourceDescriptor::Texture3d(_) => {
                full_code.push_str(&format!("    let _map_result = {};\n", map_call));

                let base_ty_str = sig.output_element_type.base_type().as_str();
                let pad_val = if base_ty_str == "f32" { "0.0" } else { "0" };
                let sig_comps = sig.output_element_type.component_count();

                let store_val = match sig_comps {
                    1 => format!(
                        "vec4<{}>(_map_result, {}, {}, {})",
                        base_ty_str, pad_val, pad_val, pad_val
                    ),
                    2 => format!(
                        "vec4<{}>(_map_result, {}, {})",
                        base_ty_str, pad_val, pad_val
                    ),
                    4 => "_map_result".to_string(),
                    _ => "_map_result".to_string(),
                };

                full_code.push_str(&format!(
                    "    textureStore(output, _global_index, {});\n",
                    store_val
                ));
            }
        }
        full_code.push_str("}\n");

        let shader = ComputeShader::new(context, full_code)?;
        let pipeline = ComputePipeline::new(context, shader)?;
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

pub struct Map;

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
        let sig = MapDefinition::parse_signature(&definition.code)?;

        let input_descriptor = if let Some(input) = input {
            if sig.user_params.is_empty() {
                None
            } else {
                Some(ResourceDescriptor::from_resource(
                    input,
                    sig.user_params[0].1.clone(),
                ))
            }
        } else {
            None
        };

        let output_descriptor = ResourceDescriptor::from_resource(output, sig.output_element_type);

        let mut secondary_resources = HashMap::new();
        if let Some(extra) = extra_parameters.as_ref() {
            for (name, val) in &extra.parameters {
                match val {
                    fabelgeist_gpu::data::gpu::parameters::PassParameter::Buffer(_b) => {
                        if let Some((_, param_ty)) = sig.user_params.iter().find(|(n, _)| n == name)
                        {
                            secondary_resources
                                .insert(name.clone(), ResourceDescriptor::Buffer(param_ty.clone()));
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
