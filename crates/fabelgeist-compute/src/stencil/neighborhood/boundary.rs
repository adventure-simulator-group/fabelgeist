//! Native boundary-coordinate arithmetic for each supported neighborhood.

use crate::BoundaryMode;

pub(in crate::stencil) fn declare(code: &mut String, mode: BoundaryMode) {
    let get_coord_1d = if mode == BoundaryMode::Wrap {
        "fn get_coord_1d(x: i32, w: i32) -> i32 { return (x % w + w) % w; }\n"
    } else {
        "fn get_coord_1d(x: i32, w: i32) -> i32 { return clamp(x, 0, w - 1); }\n"
    };

    let get_coord_2d = if mode == BoundaryMode::Wrap {
        "fn get_coord_2d(c: vec2<i32>, dim: vec2<i32>) -> vec2<i32> { return vec2<i32>((c.x % dim.x + dim.x) % dim.x, (c.y % dim.y + dim.y) % dim.y); }\n"
    } else {
        "fn get_coord_2d(c: vec2<i32>, dim: vec2<i32>) -> vec2<i32> { return clamp(c, vec2<i32>(0), dim - 1); }\n"
    };

    let get_coord_3d = if mode == BoundaryMode::Wrap {
        "fn get_coord_3d(c: vec3<i32>, dim: vec3<i32>) -> vec3<i32> { return vec3<i32>((c.x % dim.x + dim.x) % dim.x, (c.y % dim.y + dim.y) % dim.y, (c.z % dim.z + dim.z) % dim.z); }\n"
    } else {
        "fn get_coord_3d(c: vec3<i32>, dim: vec3<i32>) -> vec3<i32> { return clamp(c, vec3<i32>(0), dim - 1); }\n"
    };

    let get_coord_4d = if mode == BoundaryMode::Wrap {
        "fn get_coord_4d(c: vec4<i32>, dim: vec4<i32>) -> vec4<i32> { return vec4<i32>((c.x % dim.x + dim.x) % dim.x, (c.y % dim.y + dim.y) % dim.y, (c.z % dim.z + dim.z) % dim.z, (c.w % dim.w + dim.w) % dim.w); }\n"
    } else {
        "fn get_coord_4d(c: vec4<i32>, dim: vec4<i32>) -> vec4<i32> { return clamp(c, vec4<i32>(0), dim - 1); }\n"
    };

    code.push_str(get_coord_1d);
    code.push_str(get_coord_2d);
    code.push_str(get_coord_3d);
    code.push_str(get_coord_4d);
    code.push('\n');
}
