//! Authored native fluid program; numerical operations are preserved.

use crate::prelude::*;

pub(super) fn source(
    mode: BoundaryMode,
    input_res: &ResourceDescriptor,
    output_res: &ResourceDescriptor,
) -> Result<String> {
    let mut full_code = String::new();

    if !matches!(
        input_res,
        ResourceDescriptor::Texture2d(_) | ResourceDescriptor::Texture3d(_)
    ) {
        return Err(anyhow!("Divergence only supports Texture2d or Texture3d"));
    }

    full_code.push_str(&input_res.to_wgsl_input_binding(0, 0, "velocity"));
    full_code.push_str(&output_res.to_wgsl_output_binding(0, 1, "output")?);

    full_code.push_str("\nstruct Parameters {\n");
    full_code.push_str("    half_inverse_cell_size: f32,\n");
    full_code.push_str("    _pad: vec3<f32>,\n");
    full_code.push_str("};\n");
    full_code.push_str("@group(0) @binding(2) var<uniform> _params: Parameters;\n\n");

    match input_res {
        ResourceDescriptor::Texture2d(_) => plane(&mut full_code, mode),
        ResourceDescriptor::Texture3d(_) => volume(&mut full_code, mode),
        _ => return Err(anyhow!("Divergence only supports Texture2d or Texture3d")),
    }
    Ok(full_code)
}

fn plane(full_code: &mut String, mode: BoundaryMode) {
    full_code.push_str("@compute @workgroup_size(16, 16, 1)\n");
    full_code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
    full_code.push_str("    let tex_dim = textureDimensions(velocity);\n");
    full_code
        .push_str("    if (global_id.x >= tex_dim.x || global_id.y >= tex_dim.y) { return; }\n");
    full_code.push_str("    let x = i32(global_id.x);\n");
    full_code.push_str("    let y = i32(global_id.y);\n");
    full_code.push_str("    let w = i32(tex_dim.x);\n");
    full_code.push_str("    let h = i32(tex_dim.y);\n");
    if mode == BoundaryMode::Wrap {
        full_code
            .push_str("    let L = textureLoad(velocity, vec2<i32>((x - 1 + w) % w, y), 0).x;\n");
        full_code.push_str("    let R = textureLoad(velocity, vec2<i32>((x + 1) % w, y), 0).x;\n");
        full_code
            .push_str("    let B = textureLoad(velocity, vec2<i32>(x, (y - 1 + h) % h), 0).y;\n");
        full_code.push_str("    let T = textureLoad(velocity, vec2<i32>(x, (y + 1) % h), 0).y;\n");
    } else {
        full_code
            .push_str("    let L = textureLoad(velocity, vec2<i32>(max(x - 1, 0), y), 0).x;\n");
        full_code
            .push_str("    let R = textureLoad(velocity, vec2<i32>(min(x + 1, w - 1), y), 0).x;\n");
        full_code
            .push_str("    let B = textureLoad(velocity, vec2<i32>(x, max(y - 1, 0)), 0).y;\n");
        full_code
            .push_str("    let T = textureLoad(velocity, vec2<i32>(x, min(y + 1, h - 1)), 0).y;\n");
    }
    full_code.push_str("    let div = (R - L + T - B) * _params.half_inverse_cell_size;\n");
    full_code.push_str("    textureStore(output, global_id.xy, vec4<f32>(div, 0.0, 0.0, 1.0));\n");
    full_code.push_str("}\n");
}

fn volume(full_code: &mut String, mode: BoundaryMode) {
    full_code.push_str("@compute @workgroup_size(8, 8, 4)\n");
    full_code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
    full_code.push_str("    let tex_dim = textureDimensions(velocity);\n");
    full_code.push_str("    if (global_id.x >= tex_dim.x || global_id.y >= tex_dim.y || global_id.z >= tex_dim.z) { return; }\n");
    full_code.push_str("    let coords = vec3<i32>(global_id.xyz);\n");
    full_code.push_str("    let dim = vec3<i32>(tex_dim);\n");
    if mode == BoundaryMode::Wrap {
        full_code.push_str(
            "    let L = textureLoad(velocity, (coords - vec3<i32>(1, 0, 0) + dim) % dim, 0).x;\n",
        );
        full_code.push_str(
            "    let R = textureLoad(velocity, (coords + vec3<i32>(1, 0, 0)) % dim, 0).x;\n",
        );
        full_code.push_str(
            "    let B = textureLoad(velocity, (coords - vec3<i32>(0, 1, 0) + dim) % dim, 0).y;\n",
        );
        full_code.push_str(
            "    let T = textureLoad(velocity, (coords + vec3<i32>(0, 1, 0)) % dim, 0).y;\n",
        );
        full_code.push_str(
            "    let F = textureLoad(velocity, (coords - vec3<i32>(0, 0, 1) + dim) % dim, 0).z;\n",
        );
        full_code.push_str(
            "    let BK = textureLoad(velocity, (coords + vec3<i32>(0, 0, 1)) % dim, 0).z;\n",
        );
    } else {
        full_code.push_str("    let L = textureLoad(velocity, clamp(coords - vec3<i32>(1, 0, 0), vec3<i32>(0), dim - 1), 0).x;\n");
        full_code.push_str("    let R = textureLoad(velocity, clamp(coords + vec3<i32>(1, 0, 0), vec3<i32>(0), dim - 1), 0).x;\n");
        full_code.push_str("    let B = textureLoad(velocity, clamp(coords - vec3<i32>(0, 1, 0), vec3<i32>(0), dim - 1), 0).y;\n");
        full_code.push_str("    let T = textureLoad(velocity, clamp(coords + vec3<i32>(0, 1, 0), vec3<i32>(0), dim - 1), 0).y;\n");
        full_code.push_str("    let F = textureLoad(velocity, clamp(coords - vec3<i32>(0, 0, 1), vec3<i32>(0), dim - 1), 0).z;\n");
        full_code.push_str("    let BK = textureLoad(velocity, clamp(coords + vec3<i32>(0, 0, 1), vec3<i32>(0), dim - 1), 0).z;\n");
    }
    full_code
        .push_str("    let div = (R - L + T - B + BK - F) * _params.half_inverse_cell_size;\n");
    full_code.push_str("    textureStore(output, global_id, vec4<f32>(div, 0.0, 0.0, 1.0));\n");
    full_code.push_str("}\n");
}
