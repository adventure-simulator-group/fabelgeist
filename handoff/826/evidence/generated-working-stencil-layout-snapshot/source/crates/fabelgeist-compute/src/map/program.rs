//! Native coordinate and result emission for the admitted Map roles.

use super::MapSignature;
use crate::prelude::*;
use crate::source_error::{GeneratedBinding, GeneratedFunction, GeneratedProgramError};
use fabelgeist_gpu::data::gpu::signature::CoordinateShape;

pub(super) fn index(
    code: &mut String,
    signature: &MapSignature,
    output: &ResourceDescriptor,
) -> Result<TextureAddressing> {
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
            if !matches!(
                output,
                ResourceDescriptor::Texture2d(_) | ResourceDescriptor::Texture3d(_)
            ) {
                return Err(GeneratedProgramError::UnsupportedCoordinate {
                    function: GeneratedFunction::Map,
                    shape: parameter.shape(),
                    binding: GeneratedBinding::Output,
                    actual: output.resource_type(),
                }
                .into());
            }
            code.push_str("    let index = _global_index.xy;\n");
        } else if parameter.shape() == CoordinateShape::Volume {
            if !matches!(output, ResourceDescriptor::Texture3d(_)) {
                return Err(GeneratedProgramError::UnsupportedCoordinate {
                    function: GeneratedFunction::Map,
                    shape: parameter.shape(),
                    binding: GeneratedBinding::Output,
                    actual: output.resource_type(),
                }
                .into());
            }
            code.push_str("    let index = _global_index;\n");
        }
    }

    Ok(addressing)
}

pub(super) fn size(
    code: &mut String,
    signature: &MapSignature,
    output: &ResourceDescriptor,
) -> TextureAddressing {
    let mut addressing = TextureAddressing::Coordinates;
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

pub(super) fn store(
    code: &mut String,
    signature: &MapSignature,
    output: &ResourceDescriptor,
    map_call: &str,
) -> Result<()> {
    // Output logic
    match output {
        ResourceDescriptor::Buffer(_) => {
            code.push_str(&format!("    output[_global_index] = {};\n", map_call));
        }
        ResourceDescriptor::Texture2d(_) | ResourceDescriptor::Texture3d(_) => {
            code.push_str(&format!("    let _map_result = {};\n", map_call));

            let store_val = TextureValue::try_from(signature.output_element_type())?
                .store_expression("_map_result");

            code.push_str(&format!(
                "    textureStore(output, _global_index, {});\n",
                store_val
            ));
        }
    }
    Ok(())
}
