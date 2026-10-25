use crate::prelude::*;
use fabelgeist_gpu::data::gpu::ComputeEntryPoint::Named;
use fabelgeist_gpu::data::gpu::signature::{CoordinateParameter, CoordinateShape};
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use std::sync::Arc;

type ScatterPipelineCache = ComputeSourceCache<ScatterRecipe, Arc<ResourcePipeline>>;

#[derive(Clone, Debug, PartialEq)]
pub struct ScatterSignature {
    pub scatter_args: Vec<String>, // Raw arguments
    pub input_element_type: ResourceBaseType,
    pub output_element_type: ResourceBaseType,
    pub index: Option<CoordinateParameter>,
    pub param_names: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ScatterRecipe {
    input: ResourceDescriptor,
    output: ResourceDescriptor,
}

#[derive(Clone, Debug)]
pub struct ScatterDefinition {
    pub code: String,
    cache: ScatterPipelineCache,
}

impl ScatterSignature {
    fn call_arguments(&self) -> Vec<String> {
        let mut arguments = Vec::new();
        for (name, declaration) in self.param_names.iter().zip(&self.scatter_args) {
            // parse_signature admits exactly two colon-separated parts.
            let parts: Vec<_> = declaration.split(':').collect();
            let parameter_type = parts[1];
            if self.index.as_ref().map(CoordinateParameter::name) == Some(name.as_str()) {
                arguments.push("index".to_string());
            } else if !parameter_type.contains("Resource")
                && !parameter_type.contains("ptr<")
                && !parameter_type.contains("texture_")
            {
                arguments.push("in_val".to_string());
            }
        }
        arguments
    }
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

        let args_str = &caps[1];

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

            let index_shape = CoordinateShape::from_native_type(ty_str);
            let is_resource = ty_str.contains("Resource")
                || ty_str.contains("ptr<")
                || ty_str.contains("texture_");

            if let Some(shape) = index_shape.filter(|_| !is_resource) {
                if name == "index" || index.is_none() {
                    index = Some(CoordinateParameter::new(name, shape));
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
            index,
            param_names,
        })
    }

    pub fn build_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceDescriptor,
        output_res: ResourceDescriptor,
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

        full_code.push_str(&input_res.to_wgsl_input_binding(0, 0, "input"));
        full_code.push_str(&output_res.to_wgsl_output_binding(0, 1, "output")?);

        full_code.push('\n');
        full_code.push_str(&transformed_code);
        full_code.push('\n');

        // Threads driven by INPUT resolution
        match input_res {
            ResourceDescriptor::Buffer(_) => full_code.push_str("@compute @workgroup_size(64)\n"),
            ResourceDescriptor::Texture2d(_) => full_code.push_str("@compute @workgroup_size(16, 16)\n"),
            ResourceDescriptor::Texture3d(_) => full_code.push_str("@compute @workgroup_size(4, 4, 4)\n"),
        }
        full_code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");

        match input_res {
            ResourceDescriptor::Buffer(_) => {
                full_code.push_str("    let _global_index = global_id.x;\n");
                full_code.push_str("    if (_global_index >= arrayLength(&input)) { return; }\n");
                full_code.push_str("    let in_val = input[_global_index];\n");
            }
            ResourceDescriptor::Texture2d(_) => {
                full_code.push_str("    let _global_index = global_id.xy;\n");
                full_code.push_str("    let tex_dim = textureDimensions(input);\n");
                full_code.push_str("    if (_global_index.x >= tex_dim.x || _global_index.y >= tex_dim.y) { return; }\n");
                let suffix = TextureValue::try_from(&sig.input_element_type)?.load_suffix();
                full_code.push_str(&format!(
                    "    let in_val = textureLoad(input, _global_index, 0){suffix};\n"
                ));
            }
            ResourceDescriptor::Texture3d(_) => {
                full_code.push_str("    let _global_index = global_id;\n");
                full_code.push_str("    let tex_dim = textureDimensions(input);\n");
                full_code.push_str("    if (_global_index.x >= tex_dim.x || _global_index.y >= tex_dim.y || _global_index.z >= tex_dim.z) { return; }\n");
                let suffix = TextureValue::try_from(&sig.input_element_type)?.load_suffix();
                full_code.push_str(&format!(
                    "    let in_val = textureLoad(input, _global_index, 0){suffix};\n"
                ));
            }
        }

        if let Some(parameter) = &sig.index {
            if parameter.shape() == CoordinateShape::Scalar {
                addressing = addressing.combine(TextureAddressing::LinearIndex);
                match input_res {
                    ResourceDescriptor::Buffer(_) => full_code.push_str("    let index = _global_index;\n"),
                    ResourceDescriptor::Texture2d(_) => full_code.push_str("    let index = _global_index.y * tex_dim.x + _global_index.x;\n"),
                    ResourceDescriptor::Texture3d(_) => full_code.push_str("    let index = (_global_index.z * tex_dim.y + _global_index.y) * tex_dim.x + _global_index.x;\n"),
                }
            } else if parameter.shape() == CoordinateShape::Plane {
                if matches!(input_res, ResourceDescriptor::Buffer(_)) {
                    return Err(anyhow::anyhow!("vec2 index not supported for Buffer"));
                }
                full_code.push_str("    let index = _global_index.xy;\n");
            } else if parameter.shape() == CoordinateShape::Volume {
                if !matches!(input_res, ResourceDescriptor::Texture3d(_)) {
                    return Err(anyhow::anyhow!("vec3 index only supported for Texture3d"));
                }
                full_code.push_str("    let index = _global_index;\n");
            }
        }

        let scatter_call = format!("scatter({})", sig.call_arguments().join(", "));

        full_code.push_str(&format!("    {};\n", scatter_call));
        full_code.push_str("}\n");

        let shader = ComputeShader::new(context, full_code)?;
        let pipeline = ComputePipeline::new(context, shader, Named("main"))?;
        ResourcePipeline::new(pipeline, DispatchResource::Input, addressing)
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

pub struct Scatter;

impl Scatter {
    pub fn execute(
        context: &WgpuContext,
        definition: &ScatterDefinition,
        input: &GpuResource,
        output: &GpuResource,
    ) -> Result<()> {
        let signature = ScatterDefinition::parse_signature(&definition.code)?;
        let pipeline_info = definition.get_or_create_pipeline(
            context,
            ResourceDescriptor::from_resource(input, signature.input_element_type),
            ResourceDescriptor::from_resource(output, signature.output_element_type),
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
