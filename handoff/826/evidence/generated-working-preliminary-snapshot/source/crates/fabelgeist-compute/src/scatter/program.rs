//! Native input loading and coordinate emission for Scatter.

use super::ScatterSignature;
use crate::prelude::*;
use fabelgeist_gpu::data::gpu::ComputeEntryPoint::Named;
use fabelgeist_gpu::data::gpu::signature::CoordinateShape;

pub(super) fn build(
    context: &WgpuContext,
    source: &str,
    signature: &ScatterSignature,
    input: ResourceDescriptor,
    output: ResourceDescriptor,
) -> Result<ResourcePipeline> {
    let re = regex::Regex::new(r"(?s)fn\s+scatter\s*\(([^)]*)\)")?;
    let transformed = re.replace(
        source,
        format!(
            "fn scatter({})",
            signature.declarations().collect::<Vec<_>>().join(", ")
        ),
    );
    let mut code = String::new();
    code.push_str(&input.to_wgsl_input_binding(0, 0, "input"));
    code.push_str(&output.to_wgsl_output_binding(0, 1, "output")?);
    code.push('\n');
    code.push_str(&transformed);
    code.push('\n');
    match input {
        ResourceDescriptor::Buffer(_) => code.push_str("@compute @workgroup_size(64)\n"),
        ResourceDescriptor::Texture2d(_) => code.push_str("@compute @workgroup_size(16, 16)\n"),
        ResourceDescriptor::Texture3d(_) => code.push_str("@compute @workgroup_size(4, 4, 4)\n"),
    }
    code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
    load(&mut code, signature, &input)?;
    let addressing = index(&mut code, signature, &input)?;
    let call = format!("scatter({})", signature.call_arguments().join(", "));
    code.push_str(&format!("    {};\n", call));
    code.push_str("}\n");
    let shader = ComputeShader::new(context, code)?;
    let pipeline = ComputePipeline::new(context, shader, Named("main"))?;
    ResourcePipeline::new(pipeline, DispatchResource::Input, addressing)
}

fn load(
    code: &mut String,
    signature: &ScatterSignature,
    input: &ResourceDescriptor,
) -> TextureValueResult<()> {
    match input {
        ResourceDescriptor::Buffer(_) => {
            code.push_str("    let _global_index = global_id.x;\n");
            code.push_str("    if (_global_index >= arrayLength(&input)) { return; }\n");
            code.push_str("    let in_val = input[_global_index];\n");
        }
        ResourceDescriptor::Texture2d(_) => {
            code.push_str("    let _global_index = global_id.xy;\n");
            code.push_str("    let tex_dim = textureDimensions(input);\n");
            code.push_str("    if (_global_index.x >= tex_dim.x || _global_index.y >= tex_dim.y) { return; }\n");
            let suffix = TextureValue::try_from(signature.input_element_type())?.load_suffix();
            code.push_str(&format!(
                "    let in_val = textureLoad(input, _global_index, 0){suffix};\n"
            ));
        }
        ResourceDescriptor::Texture3d(_) => {
            code.push_str("    let _global_index = global_id;\n");
            code.push_str("    let tex_dim = textureDimensions(input);\n");
            code.push_str("    if (_global_index.x >= tex_dim.x || _global_index.y >= tex_dim.y || _global_index.z >= tex_dim.z) { return; }\n");
            let suffix = TextureValue::try_from(signature.input_element_type())?.load_suffix();
            code.push_str(&format!(
                "    let in_val = textureLoad(input, _global_index, 0){suffix};\n"
            ));
        }
    }

    Ok(())
}

fn index(
    code: &mut String,
    signature: &ScatterSignature,
    input: &ResourceDescriptor,
) -> Result<TextureAddressing> {
    let mut addressing = TextureAddressing::Coordinates;
    if let Some(parameter) = signature.index() {
        if parameter.shape() == CoordinateShape::Scalar {
            addressing = addressing.combine(TextureAddressing::LinearIndex);
            match input {
                    ResourceDescriptor::Buffer(_) => code.push_str("    let index = _global_index;\n"),
                    ResourceDescriptor::Texture2d(_) => code.push_str("    let index = _global_index.y * tex_dim.x + _global_index.x;\n"),
                    ResourceDescriptor::Texture3d(_) => code.push_str("    let index = (_global_index.z * tex_dim.y + _global_index.y) * tex_dim.x + _global_index.x;\n"),
                }
        } else if parameter.shape() == CoordinateShape::Plane {
            if matches!(input, ResourceDescriptor::Buffer(_)) {
                return Err(anyhow::anyhow!("vec2 index not supported for Buffer"));
            }
            code.push_str("    let index = _global_index.xy;\n");
        } else if parameter.shape() == CoordinateShape::Volume {
            if !matches!(input, ResourceDescriptor::Texture3d(_)) {
                return Err(anyhow::anyhow!("vec3 index only supported for Texture3d"));
            }
            code.push_str("    let index = _global_index;\n");
        }
    }

    Ok(addressing)
}
