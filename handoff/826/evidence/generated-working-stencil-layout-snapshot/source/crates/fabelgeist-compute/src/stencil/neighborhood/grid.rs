//! Native declarations for the existing neighborhood-grid policy.

use super::NeighborhoodDimension;
use super::layout::DEFAULT_NEIGHBORHOOD_SIDE;
use crate::prelude::*;
use crate::stencil::signature::StencilSignature;
use fabelgeist_gpu::data::gpu::signature::CoordinateShape;

pub(in crate::stencil) fn declaration(
    dimension: NeighborhoodDimension,
    signature: &StencilSignature,
    input: &ResourceDescriptor,
    output: &ResourceDescriptor,
) -> String {
    match dimension {
        NeighborhoodDimension::One => line(signature, input),
        NeighborhoodDimension::Two => plane(signature, input, output),
        NeighborhoodDimension::Three => volume(signature, input, output),
        NeighborhoodDimension::Four => four(signature),
    }
}

fn line(signature: &StencilSignature, input: &ResourceDescriptor) -> String {
    match input {
        ResourceDescriptor::Texture2d(_) => {
            "    let _grid_w = i32(textureDimensions(input).x);\n".to_string()
        }
        ResourceDescriptor::Texture3d(_) => {
            "    let _grid_w = i32(textureDimensions(input).x);\n".to_string()
        }
        ResourceDescriptor::Buffer(_) => {
            if signature.size().is_some() {
                "    let _grid_w = i32(size);\n".to_string()
            } else {
                "    let _grid_w = i32(arrayLength(&input));\n".to_string()
            }
        }
    }
}

fn plane(
    signature: &StencilSignature,
    input: &ResourceDescriptor,
    output: &ResourceDescriptor,
) -> String {
    match input {
        ResourceDescriptor::Texture2d(_) => {
            "    let _grid_dim = vec2<i32>(textureDimensions(input));\n".to_string()
        }
        ResourceDescriptor::Texture3d(_) => {
            "    let _grid_dim = vec2<i32>(textureDimensions(input).xy);\n".to_string()
        }
        ResourceDescriptor::Buffer(_) => {
            if let Some(parameter) = signature.size() {
                if parameter.shape() == CoordinateShape::Plane {
                    "    let _grid_dim = vec2<i32>(size);\n".to_string()
                } else if parameter.shape() == CoordinateShape::Volume {
                    "    let _grid_dim = vec2<i32>(size.xy);\n".to_string()
                } else {
                    "    let _grid_dim = vec2<i32>(i32(size), 1);\n".to_string()
                }
            } else {
                match output {
                    ResourceDescriptor::Texture2d(_) => {
                        "    let _grid_dim = vec2<i32>(textureDimensions(output));\n".to_string()
                    }
                    ResourceDescriptor::Texture3d(_) => {
                        "    let _grid_dim = vec2<i32>(textureDimensions(output).xy);\n".to_string()
                    }
                    _ => format!(
                        "    let _grid_dim = vec2<i32>({0}, {0});\n",
                        DEFAULT_NEIGHBORHOOD_SIDE
                    ),
                }
            }
        }
    }
}

fn volume(
    signature: &StencilSignature,
    input: &ResourceDescriptor,
    output: &ResourceDescriptor,
) -> String {
    match input {
        ResourceDescriptor::Texture2d(_) => {
            "    let _grid_dim = vec3<i32>(textureDimensions(input).xy, 1);\n".to_string()
        }
        ResourceDescriptor::Texture3d(_) => {
            "    let _grid_dim = vec3<i32>(textureDimensions(input));\n".to_string()
        }
        ResourceDescriptor::Buffer(_) => {
            if let Some(parameter) = signature.size() {
                if parameter.shape() == CoordinateShape::Volume {
                    "    let _grid_dim = vec3<i32>(size);\n".to_string()
                } else if parameter.shape() == CoordinateShape::Plane {
                    "    let _grid_dim = vec3<i32>(size, 1);\n".to_string()
                } else {
                    "    let _grid_dim = vec3<i32>(i32(size), 1, 1);\n".to_string()
                }
            } else {
                match output {
                    ResourceDescriptor::Texture3d(_) => {
                        "    let _grid_dim = vec3<i32>(textureDimensions(output));\n".to_string()
                    }
                    _ => format!(
                        "    let _grid_dim = vec3<i32>({0}, {0}, {0});\n",
                        DEFAULT_NEIGHBORHOOD_SIDE
                    ),
                }
            }
        }
    }
}

fn four(signature: &StencilSignature) -> String {
    {
        if let Some(parameter) = signature.size() {
            if parameter.shape() == CoordinateShape::Volume {
                "    let _grid_dim = vec4<i32>(size, 1);\n".to_string()
            } else {
                "    let _grid_dim = vec4<i32>(i32(size), 1, 1, 1);\n".to_string()
            }
        } else {
            format!(
                "    let _grid_dim = vec4<i32>({0}, {0}, {0}, {0});\n",
                DEFAULT_NEIGHBORHOOD_SIDE
            )
        }
    }
}
