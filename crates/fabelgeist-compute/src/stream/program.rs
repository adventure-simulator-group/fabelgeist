//! Native bindings and prefix-controlled Stream source emission.

use super::StreamSignature;
use crate::prelude::*;
use fabelgeist_gpu::data::gpu::ComputeEntryPoint::Named;

pub(super) fn build(
    context: &WgpuContext,
    source: &str,
    signature: &StreamSignature,
    input: ResourceDescriptor,
) -> Result<ResourcePipeline> {
    let re = regex::Regex::new(r"(?s)fn\s+stream\s*\(([^)]*)\)")?;
    let transformed = re.replace(
        source,
        format!(
            "fn stream({})",
            signature.declarations().collect::<Vec<_>>().join(", ")
        ),
    );
    let mut code = String::new();
    bindings(&mut code, signature, &input);
    code.push('\n');
    code.push_str(&transformed);
    code.push('\n');
    match input {
        ResourceDescriptor::Buffer(_) => code.push_str("@compute @workgroup_size(64)\n"),
        ResourceDescriptor::Texture2d(_) => code.push_str("@compute @workgroup_size(16, 16)\n"),
        ResourceDescriptor::Texture3d(_) => code.push_str("@compute @workgroup_size(4, 4, 4)\n"),
    }
    code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
    prefix(&mut code, &input);
    load(&mut code, signature, &input)?;
    let arguments = signature.call_arguments(&input)?;
    code.push_str(&format!("    stream({});\n", arguments.join(", ")));
    code.push_str("}\n");
    let shader = ComputeShader::new(context, code)?;
    let pipeline = ComputePipeline::new(context, shader, Named("main"))?;
    let resource = match input {
        ResourceDescriptor::Buffer(_) => DispatchResource::InclusiveOffsets,
        _ => DispatchResource::Input,
    };
    ResourcePipeline::new(pipeline, resource, TextureAddressing::LinearIndex)
}

fn bindings(code: &mut String, signature: &StreamSignature, input: &ResourceDescriptor) {
    // Globals
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

    code.push_str("@group(0) @binding(1) var<storage, read> counts: array<u32>;\n");
    code.push_str("@group(0) @binding(2) var<storage, read> inclusive_offsets: array<u32>;\n");
    code.push_str(&format!(
        "@group(0) @binding(3) var<storage, read_write> output: array<{}>;\n",
        signature.output_element_type().as_str()
    ));
}

fn prefix(code: &mut String, input: &ResourceDescriptor) {
    match input {
        ResourceDescriptor::Buffer(_) => {
            code.push_str("    let _global_index = global_id.x;\n");
            code.push_str(
                "    if (_global_index >= arrayLength(&inclusive_offsets)) { return; }\n",
            );
        }
        ResourceDescriptor::Texture2d(_) => {
            code.push_str("    let _global_index = global_id.xy;\n");
            code.push_str("    let tex_dim = textureDimensions(input);\n");
            code.push_str("    if (_global_index.x >= tex_dim.x || _global_index.y >= tex_dim.y) { return; }\n");
        }
        ResourceDescriptor::Texture3d(_) => {
            code.push_str("    let _global_index = global_id;\n");
            code.push_str("    let tex_dim = textureDimensions(input);\n");
            code.push_str("    if (_global_index.x >= tex_dim.x || _global_index.y >= tex_dim.y || _global_index.z >= tex_dim.z) { return; }\n");
        }
    }

    match input {
            ResourceDescriptor::Buffer(_) => code.push_str("    let _linear_index = _global_index;\n"),
            ResourceDescriptor::Texture2d(_) => code.push_str("    let _linear_index = _global_index.y * tex_dim.x + _global_index.x;\n"),
            ResourceDescriptor::Texture3d(_) => code.push_str("    let _linear_index = (_global_index.z * tex_dim.y + _global_index.y) * tex_dim.x + _global_index.x;\n"),
        }

    code.push_str("    let _count = counts[_linear_index];\n");
    code.push_str("    if (_count == 0u) { return; }\n");
    code.push_str("    let _offset = inclusive_offsets[_linear_index] - _count;\n");
}

fn load(
    code: &mut String,
    signature: &StreamSignature,
    input: &ResourceDescriptor,
) -> TextureValueResult<()> {
    if signature.has_input_value() {
        match input {
            ResourceDescriptor::Buffer(_) => {
                code.push_str("    let in_val = input[_linear_index];\n")
            }
            _ => {
                let swizzle = TextureValue::try_from(signature.input_element_type())?.load_suffix();
                code.push_str(&format!(
                    "    let in_val = textureLoad(input, _global_index, 0){};\n",
                    swizzle
                ));
            }
        }
    }

    Ok(())
}
