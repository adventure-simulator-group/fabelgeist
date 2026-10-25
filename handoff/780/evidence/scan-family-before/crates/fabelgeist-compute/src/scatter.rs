use crate::prelude::*;
use fabelgeist_gpu::data::gpu::resource::{GpuResource, ResourceType};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct ScatterSignature {
    pub scatter_args: Vec<String>, // Raw arguments
    pub input_element_type: ResourceBaseType,
    pub output_element_type: ResourceBaseType,
    pub index_type: Option<String>,
    pub index_param_name: Option<String>,
    pub param_names: Vec<String>,
}

type ScatterPipelineCache =
    ComputeSourceCache<(ResourceType, ResourceType), Arc<ResourcePipeline>>;

#[derive(Clone, Debug)]
pub struct ScatterDefinition {
    pub code: String,
    cache: ScatterPipelineCache,
}

impl Default for ScatterDefinition {
    fn default() -> Self {
        Self {
            code: String::new(),
            cache: ComputeSourceCache::default(),
        }
    }
}

impl PartialEq for ScatterDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl ScatterDefinition {
    pub fn new(_context: &WgpuContext, code: String) -> Result<Self> {
        let _ = Self::parse_signature(&code)?;
        Ok(Self {
            code,
            cache: ComputeSourceCache::default(),
        })
    }

    pub fn parse_signature(code: &str) -> Result<ScatterSignature> {
        let re = regex::Regex::new(r"(?s)fn\s+scatter\s*\(([^)]*)\)\s*\{")?;
        let caps = re
            .captures(code)
            .ok_or_else(|| anyhow::anyhow!("Compute shader must define a 'scatter' function"))?;

        let args_str = caps.get(1).unwrap().as_str();

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

        let mut index_type = None;
        let mut index_param_name = None;
        let mut param_names = vec![];
        let mut input_element_type = ResourceBaseType::F32;
        let mut output_element_type = ResourceBaseType::F32;
        let mut has_value_param = false;

        for arg in &args {
            let parts: Vec<&str> = arg.split(':').map(|s| s.trim()).collect();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!("Invalid scatter parameter format: {}", arg));
            }
            let name = parts[0];
            let ty_str = parts[1];

            param_names.push(name.to_string());

            let is_index_type = ty_str == "u32" || ty_str == "vec2<u32>" || ty_str == "vec3<u32>";
            let is_resource = ty_str.contains("Resource")
                || ty_str.contains("ptr<")
                || ty_str.contains("texture_");

            if is_index_type && !is_resource {
                if name == "index" || index_param_name.is_none() {
                    index_type = Some(ty_str.to_string());
                    index_param_name = Some(name.to_string());
                }
            } else if is_resource {
                output_element_type = fabelgeist_gpu::data::gpu::signature::parse_base_type(ty_str);
            } else {
                has_value_param = true;
                input_element_type = fabelgeist_gpu::data::gpu::signature::parse_base_type(ty_str);
            }
        }

        if !has_value_param {
            return Err(anyhow::anyhow!(
                "Scatter function must take a value parameter from the input domain."
            ));
        }

        Ok(ScatterSignature {
            scatter_args: args,
            input_element_type,
            output_element_type,
            index_type,
            index_param_name,
            param_names,
        })
    }

    pub fn build_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceType,
        output_res: ResourceType,
    ) -> Result<ResourcePipeline> {
        let sig = Self::parse_signature(&self.code)?;
        let mut addressing = TextureAddressing::Coordinates;

        // 1. Strip the Resource parameter from the function signature in the code
        let mut transformed_code = self.code.clone();
        let mut new_args = Vec::new();
        for arg in &sig.scatter_args {
            let parts: Vec<&str> = arg.split(':').map(|s| s.trim()).collect();
            let name = parts[0];
            let ty = parts[1].to_string();

            if ty.contains("Resource<") || ty.contains("ptr<") || ty.contains("texture_") {
                continue;
            }
            new_args.push(format!("{}: {}", name, ty));
        }

        let re_sig = regex::Regex::new(r"(?s)fn\s+scatter\s*\(([^)]*)\)")?;
        transformed_code = re_sig
            .replace(
                &transformed_code,
                format!("fn scatter({})", new_args.join(", ")),
            )
            .to_string();

        let mut full_code = String::new();

        match input_res {
            ResourceType::Buffer => full_code.push_str(&format!(
                "@group(0) @binding(0) var<storage, read> input: array<{}>;\n",
                sig.input_element_type.as_str()
            )),
            ResourceType::Texture2d => full_code.push_str(&format!(
                "@group(0) @binding(0) var input: texture_2d<{}>;\n",
                sig.input_element_type.as_str()
            )),
            ResourceType::Texture3d => full_code.push_str(&format!(
                "@group(0) @binding(0) var input: texture_3d<{}>;\n",
                sig.input_element_type.as_str()
            )),
        }

        match output_res {
            ResourceType::Buffer => full_code.push_str(&format!(
                "@group(0) @binding(1) var<storage, read_write> output: array<{}>;\n",
                sig.output_element_type.as_str()
            )),
            ResourceType::Texture2d => full_code.push_str(
                "@group(0) @binding(1) var output: texture_storage_2d<rgba32float, write>;\n",
            ),
            ResourceType::Texture3d => full_code.push_str(
                "@group(0) @binding(1) var output: texture_storage_3d<rgba32float, write>;\n",
            ),
        }

        full_code.push('\n');
        full_code.push_str(&transformed_code);
        full_code.push('\n');

        // Threads driven by INPUT resolution
        match input_res {
            ResourceType::Buffer => full_code.push_str("@compute @workgroup_size(64)\n"),
            ResourceType::Texture2d => full_code.push_str("@compute @workgroup_size(16, 16)\n"),
            ResourceType::Texture3d => full_code.push_str("@compute @workgroup_size(4, 4, 4)\n"),
        }
        full_code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");

        match input_res {
            ResourceType::Buffer => {
                full_code.push_str("    let _global_index = global_id.x;\n");
                full_code.push_str("    if (_global_index >= arrayLength(&input)) { return; }\n");
                full_code.push_str("    let in_val = input[_global_index];\n");
            }
            ResourceType::Texture2d => {
                full_code.push_str("    let _global_index = global_id.xy;\n");
                full_code.push_str("    let tex_dim = textureDimensions(input);\n");
                full_code.push_str("    if (_global_index.x >= tex_dim.x || _global_index.y >= tex_dim.y) { return; }\n");
                if sig.input_element_type.is_scalar() {
                    full_code
                        .push_str("    let in_val = textureLoad(input, _global_index, 0).x;\n");
                } else {
                    full_code.push_str("    let in_val = textureLoad(input, _global_index, 0);\n");
                }
            }
            ResourceType::Texture3d => {
                full_code.push_str("    let _global_index = global_id;\n");
                full_code.push_str("    let tex_dim = textureDimensions(input);\n");
                full_code.push_str("    if (_global_index.x >= tex_dim.x || _global_index.y >= tex_dim.y || _global_index.z >= tex_dim.z) { return; }\n");
                if sig.input_element_type.is_scalar() {
                    full_code
                        .push_str("    let in_val = textureLoad(input, _global_index, 0).x;\n");
                } else {
                    full_code.push_str("    let in_val = textureLoad(input, _global_index, 0);\n");
                }
            }
        }

        if let Some(ref idx_ty) = sig.index_type {
            if idx_ty == "u32" {
                addressing = addressing.combine(TextureAddressing::LinearIndex);
                match input_res {
                    ResourceType::Buffer => full_code.push_str("    let index = _global_index;\n"),
                    ResourceType::Texture2d => full_code.push_str("    let index = _global_index.y * tex_dim.x + _global_index.x;\n"),
                    ResourceType::Texture3d => full_code.push_str("    let index = (_global_index.z * tex_dim.y + _global_index.y) * tex_dim.x + _global_index.x;\n"),
                }
            } else if idx_ty == "vec2<u32>" {
                if input_res == ResourceType::Buffer {
                    return Err(anyhow::anyhow!("vec2 index not supported for Buffer"));
                }
                full_code.push_str("    let index = _global_index.xy;\n");
            } else if idx_ty == "vec3<u32>" {
                if input_res != ResourceType::Texture3d {
                    return Err(anyhow::anyhow!("vec3 index only supported for Texture3d"));
                }
                full_code.push_str("    let index = _global_index;\n");
            }
        }

        let mut call_args = vec![];
        for name in &sig.param_names {
            if Some(name) == sig.index_param_name.as_ref() {
                call_args.push("index".to_string());
            } else if !sig
                .scatter_args
                .iter()
                .find(|a| a.starts_with(&format!("{}:", name)))
                .unwrap()
                .contains("Resource")
            {
                // If it is NOT the resource parameter (which got stripped), it must be the input value
                call_args.push("in_val".to_string());
            }
        }
        let scatter_call = format!("scatter({})", call_args.join(", "));

        full_code.push_str(&format!("    {};\n", scatter_call));
        full_code.push_str("}\n");

        ResourcePipeline::new(context, full_code, DispatchResource::Input, addressing)
    }

    pub fn get_or_create_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceType,
        output_res: ResourceType,
    ) -> Result<Arc<ResourcePipeline>> {
        self.cache
            .get_or_try_insert(context, &self.code, (input_res, output_res), || {
                let pipeline_info = self.build_pipeline(context, input_res, output_res)?;
                let arc_info = Arc::new(pipeline_info);
                Ok(arc_info)
            })
    }
}

pub struct Scatter;

impl Scatter {
    pub fn execute(
        context: &WgpuContext,
        definition: &ScatterDefinition,
        input: &GpuResource,
        output: &GpuResource,
    ) -> Result<()> {
        let pipeline_info = definition.get_or_create_pipeline(
            context,
            input.resource_type(),
            output.resource_type(),
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
