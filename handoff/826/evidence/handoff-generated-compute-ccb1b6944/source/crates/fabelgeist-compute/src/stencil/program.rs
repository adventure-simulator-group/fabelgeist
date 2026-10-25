//! Assemble one Stencil native program from its current source and bindings.

use super::StencilDefinition;
use super::bindings::{self, StencilBindings};
use super::neighborhood::boundary;
use super::signature::StencilSignature;
use crate::prelude::*;
use fabelgeist_gpu::data::gpu::ComputeEntryPoint::Named;
use fabelgeist_gpu::data::gpu::signature::CoordinateShape;
use std::collections::HashMap;

pub(super) fn build(
    context: &WgpuContext,
    definition: &StencilDefinition,
    input: ResourceDescriptor,
    output: ResourceDescriptor,
    secondary: &HashMap<String, ResourceDescriptor>,
) -> Result<ResourcePipeline> {
    let signature = StencilSignature::parse(&definition.code)?;
    let bindings = StencilBindings::new(&signature, secondary);
    let source = bindings::preprocess(&definition.code, &signature, secondary)?;
    let mut code = String::new();
    if let Some(neighborhood) = signature.neighborhood() {
        neighborhood.declare(&mut code);
    }
    bindings.declarations(&mut code, &input, &output, secondary)?;
    if signature.neighborhood().is_some() {
        boundary::declare(&mut code, definition.boundary_mode);
    }
    code.push_str(&source);
    code.push('\n');
    match output {
        ResourceDescriptor::Buffer(_) => code.push_str("@compute @workgroup_size(64, 1, 1)\n"),
        ResourceDescriptor::Texture2d(_) => code.push_str("@compute @workgroup_size(16, 16, 1)\n"),
        ResourceDescriptor::Texture3d(_) => code.push_str("@compute @workgroup_size(8, 8, 4)\n"),
    }
    code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
    code.push_str(&output.generate_prologue());
    let mut addressing = coordinates(&mut code, &signature, &output);
    if let Some(neighborhood) = signature.neighborhood() {
        addressing = addressing.combine(neighborhood.emit(&mut code, &signature, &input, &output)?);
    }
    bindings.unpack(&mut code, &signature);
    let call = format!(
        "stencil({})",
        bindings::call_arguments(&signature, secondary).join(", ")
    );
    store(&mut code, &signature, &output, &call)?;
    code.push_str("}\n");
    let shader = ComputeShader::new(context, code)?;
    let pipeline = ComputePipeline::new(context, shader, Named("main"))?;
    ResourcePipeline::new(pipeline, DispatchResource::Output, addressing)
}

fn coordinates(
    code: &mut String,
    signature: &StencilSignature,
    output: &ResourceDescriptor,
) -> TextureAddressing {
    let mut addressing = TextureAddressing::Coordinates;
    if let Some(parameter) = signature.index() {
        if parameter.shape() == CoordinateShape::Scalar {
            addressing = addressing.combine(TextureAddressing::LinearIndex);
            match output {
                    ResourceDescriptor::Buffer(_) => code.push_str("    let index = _global_index;\n"),
                    ResourceDescriptor::Texture2d(_) => code.push_str("    let index = _global_index.y * tex_dim.x + _global_index.x;\n"),
                    ResourceDescriptor::Texture3d(_) => code.push_str("    let index = (_global_index.z * tex_dim.y + _global_index.y) * tex_dim.x + _global_index.x;\n"),
                }
        } else if parameter.shape() == CoordinateShape::Plane {
            code.push_str("    let index = _global_index.xy;\n");
        } else if parameter.shape() == CoordinateShape::Volume {
            code.push_str("    let index = _global_index;\n");
        }
    }

    if let Some(parameter) = signature.size() {
        match parameter.shape() {
            CoordinateShape::Scalar => {
                addressing = TextureAddressing::LinearPopulation;
                match output {
                    ResourceDescriptor::Buffer(_) => {
                        code.push_str("    let size = arrayLength(&output);\n")
                    }
                    ResourceDescriptor::Texture2d(_) => {
                        code.push_str("    let size = tex_dim.x * tex_dim.y;\n")
                    }
                    ResourceDescriptor::Texture3d(_) => {
                        code.push_str("    let size = tex_dim.x * tex_dim.y * tex_dim.z;\n")
                    }
                }
            }
            CoordinateShape::Plane => match output {
                ResourceDescriptor::Buffer(_) => {
                    code.push_str("    let size = vec2<u32>(arrayLength(&output), 1u);\n")
                }
                ResourceDescriptor::Texture2d(_) => code.push_str("    let size = tex_dim;\n"),
                ResourceDescriptor::Texture3d(_) => code.push_str("    let size = tex_dim.xy;\n"),
            },
            CoordinateShape::Volume => match output {
                ResourceDescriptor::Buffer(_) => {
                    code.push_str("    let size = vec3<u32>(arrayLength(&output), 1u, 1u);\n")
                }
                ResourceDescriptor::Texture2d(_) => {
                    code.push_str("    let size = vec3<u32>(tex_dim, 1u);\n")
                }
                ResourceDescriptor::Texture3d(_) => code.push_str("    let size = tex_dim;\n"),
            },
        }
    }

    addressing
}

fn store(
    code: &mut String,
    signature: &StencilSignature,
    output: &ResourceDescriptor,
    call: &str,
) -> TextureValueResult<()> {
    match output {
        ResourceDescriptor::Buffer(_) => {
            code.push_str(&format!("    output[_global_index] = {};\n", call))
        }
        ResourceDescriptor::Texture2d(_) | ResourceDescriptor::Texture3d(_) => {
            code.push_str(&format!("    let _stencil_result = {};\n", call));
            let value = TextureValue::try_from(signature.output_element_type())?
                .store_expression("_stencil_result");
            code.push_str(&format!(
                "    textureStore(output, _global_index, {});\n",
                value
            ));
        }
    }
    Ok(())
}
