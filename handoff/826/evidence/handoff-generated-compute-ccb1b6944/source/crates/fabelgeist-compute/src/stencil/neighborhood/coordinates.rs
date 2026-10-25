//! Project the launch coordinates into the admitted neighborhood grid.

use super::NeighborhoodDimension;
use crate::prelude::*;

pub(super) fn emit(
    code: &mut String,
    dimension: NeighborhoodDimension,
    output: &ResourceDescriptor,
) -> TextureAddressing {
    if matches!(dimension, NeighborhoodDimension::Four) {
        return flat_four(code, output);
    }
    match (dimension, output) {
        (NeighborhoodDimension::One, _) => {
            code.push_str("    let coords = i32(global_id.x);\n");
        }
        (NeighborhoodDimension::Two, ResourceDescriptor::Buffer(_)) => {
            code.push_str("    let coords = vec2<i32>(i32(_global_index % u32(_grid_dim.x)), i32(_global_index / u32(_grid_dim.x)));\n");
        }
        (NeighborhoodDimension::Three, ResourceDescriptor::Buffer(_)) => {
            code.push_str("    let coords = vec3<i32>(i32(_global_index % u32(_grid_dim.x)), i32((_global_index / u32(_grid_dim.x)) % u32(_grid_dim.y)), i32(_global_index / (u32(_grid_dim.x) * u32(_grid_dim.y))));\n");
        }
        (NeighborhoodDimension::Two, _) => {
            code.push_str("    let coords = vec2<i32>(global_id.xy);\n");
        }
        (NeighborhoodDimension::Three, _) => {
            code.push_str("    let coords = vec3<i32>(global_id.xyz);\n");
        }
        (NeighborhoodDimension::Four, _) => {}
    }
    TextureAddressing::Coordinates
}

fn flat_four(code: &mut String, output: &ResourceDescriptor) -> TextureAddressing {
    let addressing = match output {
        ResourceDescriptor::Buffer(_) => {
            code.push_str("    let _flat_idx = _global_index;\n");
            TextureAddressing::Coordinates
        }
        ResourceDescriptor::Texture2d(_) => {
            code.push_str("    let _flat_idx = _global_index.y * tex_dim.x + _global_index.x;\n");
            TextureAddressing::LinearIndex
        }
        ResourceDescriptor::Texture3d(_) => {
            code.push_str("    let _flat_idx = (_global_index.z * tex_dim.y + _global_index.y) * tex_dim.x + _global_index.x;\n");
            TextureAddressing::LinearIndex
        }
    };
    code.push_str("    let _coord_x = i32(_flat_idx % u32(_grid_dim.x));\n");
    code.push_str("    let _coord_y = i32((_flat_idx / u32(_grid_dim.x)) % u32(_grid_dim.y));\n");
    code.push_str("    let _coord_z = i32((_flat_idx / (u32(_grid_dim.x) * u32(_grid_dim.y))) % u32(_grid_dim.z));\n");
    code.push_str("    let _coord_w = i32(_flat_idx / (u32(_grid_dim.x) * u32(_grid_dim.y) * u32(_grid_dim.z)));\n");
    code.push_str("    let coords = vec4<i32>(_coord_x, _coord_y, _coord_z, _coord_w);\n");
    addressing
}
