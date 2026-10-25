//! Native bindings, coordinates and result emission for Gather.

use super::GatherSignature;
use crate::prelude::*;
use fabelgeist_gpu::data::gpu::ComputeEntryPoint::Named;
use fabelgeist_gpu::data::gpu::signature::CoordinateShape;

pub(super) fn build(
    context: &WgpuContext,
    source: &str,
    signature: &GatherSignature,
    input: ResourceDescriptor,
    output: ResourceDescriptor,
) -> Result<ResourcePipeline> {
    let re = regex::Regex::new(r"(?s)fn\s+gather\s*\(([^)]*)\)")?;
    let transformed = re.replace(
        source,
        format!(
            "fn gather({})",
            signature.declarations().collect::<Vec<_>>().join(", ")
        ),
    );
    let mut code = String::new();
    bindings(&mut code, signature, &input, &output)?;
    code.push('\n');
    code.push_str(&transformed);
    code.push('\n');
    match output {
        ResourceDescriptor::Buffer(_) => code.push_str("@compute @workgroup_size(64)\n"),
        ResourceDescriptor::Texture2d(_) => code.push_str("@compute @workgroup_size(16, 16)\n"),
        ResourceDescriptor::Texture3d(_) => code.push_str("@compute @workgroup_size(4, 4, 4)\n"),
    }
    code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
    code.push_str(&output.generate_prologue());
    let addressing = index(&mut code, signature, &output)?;
    let call = format!("gather({})", signature.call_arguments().join(", "));
    match output {
        ResourceDescriptor::Buffer(_) => {
            code.push_str(&format!("    output[_global_index] = {};\n", call));
        }
        ResourceDescriptor::Texture2d(_) | ResourceDescriptor::Texture3d(_) => {
            code.push_str(&format!("    let _map_result = {};\n", call));
            let value = TextureValue::try_from(signature.output_element_type())?
                .store_expression("_map_result");
            code.push_str(&format!(
                "    textureStore(output, _global_index, {});\n",
                value
            ));
        }
    }
    code.push_str("}\n");
    let shader = ComputeShader::new(context, code)?;
    let pipeline = ComputePipeline::new(context, shader, Named("main"))?;
    ResourcePipeline::new(pipeline, DispatchResource::Output, addressing)
}

fn bindings(
    code: &mut String,
    signature: &GatherSignature,
    input: &ResourceDescriptor,
    output: &ResourceDescriptor,
) -> Result<()> {
    match input {
        ResourceDescriptor::Buffer(_) => code.push_str(&format!(
            "@group(0) @binding(0) var<storage, read> input: array<{}>;\n",
            signature.input_element_type().as_str()
        )),
        ResourceDescriptor::Texture2d(_) => code.push_str(&format!(
            "@group(0) @binding(0) var input: texture_2d<{}>;\n",
            signature.input_element_type().base_type().as_str()
        )),
        ResourceDescriptor::Texture3d(_) => code.push_str(&format!(
            "@group(0) @binding(0) var input: texture_3d<{}>;\n",
            signature.input_element_type().base_type().as_str()
        )),
    }
    match output {
        ResourceDescriptor::Buffer(_) => code.push_str(&format!(
            "@group(0) @binding(1) var<storage, read_write> output: array<{}>;\n",
            signature.output_element_type().as_str()
        )),
        _ => code.push_str(&output.to_wgsl_output_binding(0, 1, "output")?),
    }
    Ok(())
}

fn index(
    code: &mut String,
    signature: &GatherSignature,
    output: &ResourceDescriptor,
) -> Result<TextureAddressing> {
    let Some(parameter) = signature.index() else {
        return Ok(TextureAddressing::Coordinates);
    };
    match parameter.shape() {
        CoordinateShape::Scalar => {
            match output {
                ResourceDescriptor::Buffer(_) => code.push_str("    let index = _global_index;\n"),
                ResourceDescriptor::Texture2d(_) => code.push_str("    let index = _global_index.y * tex_dim.x + _global_index.x;\n"),
                ResourceDescriptor::Texture3d(_) => code.push_str("    let index = (_global_index.z * tex_dim.y + _global_index.y) * tex_dim.x + _global_index.x;\n"),
            }
            return Ok(TextureAddressing::LinearIndex);
        }
        CoordinateShape::Plane => {
            if matches!(output, ResourceDescriptor::Buffer(_)) {
                return Err(anyhow!("vec2 index not supported for Buffer"));
            }
            code.push_str("    let index = _global_index.xy;\n");
        }
        CoordinateShape::Volume => {
            if !matches!(output, ResourceDescriptor::Texture3d(_)) {
                return Err(anyhow!("vec3 index only supported for Texture3d"));
            }
            code.push_str("    let index = _global_index;\n");
        }
    }
    Ok(TextureAddressing::Coordinates)
}
