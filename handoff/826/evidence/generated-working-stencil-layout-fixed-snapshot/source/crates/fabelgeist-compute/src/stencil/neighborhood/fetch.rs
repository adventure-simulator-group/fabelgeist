//! Native sample expressions, preserving texture-axis projections.

use super::{NeighborhoodDimension, NeighborhoodParameter};
use crate::prelude::*;

pub(super) struct NeighborhoodFetch {
    expression: String,
}

impl NeighborhoodFetch {
    pub(super) fn new(
        neighborhood: &NeighborhoodParameter,
        input: &ResourceDescriptor,
    ) -> TextureValueResult<Self> {
        let swizzle = match input {
            ResourceDescriptor::Buffer(_) => "",
            _ => TextureValue::try_from(neighborhood.element_type())?.load_suffix(),
        };
        let expression = match input {
                ResourceDescriptor::Buffer(_) => match neighborhood.dimension() {
                    NeighborhoodDimension::One => "input[{}]".to_string(),
                    NeighborhoodDimension::Two => "input[{0}.y * _grid_dim.x + {0}.x]".to_string(),
                    NeighborhoodDimension::Three => "input[({0}.z * _grid_dim.y + {0}.y) * _grid_dim.x + {0}.x]".to_string(),
                    NeighborhoodDimension::Four => "input[((({0}.w * _grid_dim.z + {0}.z) * _grid_dim.y + {0}.y) * _grid_dim.x) + {0}.x]".to_string(),
                },
                ResourceDescriptor::Texture2d(_) => match neighborhood.dimension() {
                    NeighborhoodDimension::One => format!("textureLoad(input, vec2<i32>({{}}, 0), 0){}", swizzle),
                    NeighborhoodDimension::Two => format!("textureLoad(input, {{}}, 0){}", swizzle),
                    NeighborhoodDimension::Three => format!("textureLoad(input, {{}}.xy, 0){}", swizzle),
                    NeighborhoodDimension::Four => format!("textureLoad(input, {{}}.xy, 0){}", swizzle),
                },
                ResourceDescriptor::Texture3d(_) => match neighborhood.dimension() {
                    NeighborhoodDimension::One => format!("textureLoad(input, vec3<i32>({{}}, 0, 0), 0){}", swizzle),
                    NeighborhoodDimension::Two => format!("textureLoad(input, vec3<i32>({{}}, 0), 0){}", swizzle),
                    NeighborhoodDimension::Three => format!("textureLoad(input, {{}}, 0){}", swizzle),
                    NeighborhoodDimension::Four => format!("textureLoad(input, {{}}.xyz, 0){}", swizzle),
                },

            };
        Ok(Self { expression })
    }

    pub(super) fn at(&self, coordinate: &str) -> String {
        self.expression
            .replace("{}", coordinate)
            .replace("{0}", coordinate)
    }
}
