//! Authored native fluid program; numerical operations are preserved.

use crate::prelude::*;

pub(super) fn source(
    mode: BoundaryMode,
    velocity_res: &ResourceDescriptor,
    pressure_res: &ResourceDescriptor,
    output_res: &ResourceDescriptor,
) -> Result<String> {
    let mut full_code = String::new();

    if !matches!(
        velocity_res,
        ResourceDescriptor::Texture2d(_) | ResourceDescriptor::Texture3d(_)
    ) {
        return Err(anyhow!("Gradient only supports Texture2d or Texture3d"));
    }

    full_code.push_str(&velocity_res.to_wgsl_input_binding(0, 0, "velocity"));
    full_code.push_str(&pressure_res.to_wgsl_input_binding(0, 1, "pressure"));
    full_code.push_str(&output_res.to_wgsl_output_binding(0, 2, "output")?);

    full_code.push_str("\nstruct Parameters {\n");
    full_code.push_str("    half_inverse_cell_size: f32,\n");
    full_code.push_str("    _pad: vec3<f32>,\n");
    full_code.push_str("};\n");
    full_code.push_str("@group(0) @binding(3) var<uniform> _params: Parameters;\n\n");

    match velocity_res {
        ResourceDescriptor::Texture2d(_) => plane(&mut full_code, mode),
        ResourceDescriptor::Texture3d(_) => volume(&mut full_code, mode),
        _ => return Err(anyhow!("Gradient only supports Texture2d or Texture3d")),
    }
    Ok(full_code)
}

fn plane(full_code: &mut String, mode: BoundaryMode) {
    full_code.push_str("@compute @workgroup_size(16, 16, 1)\n");
    full_code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
    full_code.push_str("    let tex_dim = textureDimensions(pressure);\n");
    full_code
        .push_str("    if (global_id.x >= tex_dim.x || global_id.y >= tex_dim.y) { return; }\n");
    full_code.push_str("    let x = i32(global_id.x);\n");
    full_code.push_str("    let y = i32(global_id.y);\n");
    full_code.push_str("    let w = i32(tex_dim.x);\n");
    full_code.push_str("    let h = i32(tex_dim.y);\n");

    if mode == BoundaryMode::Wrap {
        full_code
            .push_str("    let pL = textureLoad(pressure, vec2<i32>((x - 1 + w) % w, y), 0).x;\n");
        full_code.push_str("    let pR = textureLoad(pressure, vec2<i32>((x + 1) % w, y), 0).x;\n");
        full_code
            .push_str("    let pB = textureLoad(pressure, vec2<i32>(x, (y - 1 + h) % h), 0).x;\n");
        full_code.push_str("    let pT = textureLoad(pressure, vec2<i32>(x, (y + 1) % h), 0).x;\n");
    } else {
        full_code
            .push_str("    let pL = textureLoad(pressure, vec2<i32>(max(x - 1, 0), y), 0).x;\n");
        full_code.push_str(
            "    let pR = textureLoad(pressure, vec2<i32>(min(x + 1, w - 1), y), 0).x;\n",
        );
        full_code
            .push_str("    let pB = textureLoad(pressure, vec2<i32>(x, max(y - 1, 0)), 0).x;\n");
        full_code.push_str(
            "    let pT = textureLoad(pressure, vec2<i32>(x, min(y + 1, h - 1)), 0).x;\n",
        );
    }

    full_code.push_str("    let vel = textureLoad(velocity, global_id.xy, 0).xy;\n");
    full_code
        .push_str("    let grad = vec2<f32>(pR - pL, pT - pB) * _params.half_inverse_cell_size;\n");
    full_code
        .push_str("    textureStore(output, global_id.xy, vec4<f32>(vel - grad, 0.0, 1.0));\n");
    full_code.push_str("}\n");
}

fn volume(full_code: &mut String, mode: BoundaryMode) {
    full_code.push_str("@compute @workgroup_size(8, 8, 4)\n");
    full_code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
    full_code.push_str("    let tex_dim = textureDimensions(pressure);\n");
    full_code.push_str("    if (global_id.x >= tex_dim.x || global_id.y >= tex_dim.y || global_id.z >= tex_dim.z) { return; }\n");
    full_code.push_str("    let coords = vec3<i32>(global_id.xyz);\n");
    full_code.push_str("    let dim = vec3<i32>(tex_dim);\n");

    if mode == BoundaryMode::Wrap {
        full_code.push_str(
            "    let pL = textureLoad(pressure, (coords - vec3<i32>(1, 0, 0) + dim) % dim, 0).x;\n",
        );
        full_code.push_str(
            "    let pR = textureLoad(pressure, (coords + vec3<i32>(1, 0, 0)) % dim, 0).x;\n",
        );
        full_code.push_str(
            "    let pB = textureLoad(pressure, (coords - vec3<i32>(0, 1, 0) + dim) % dim, 0).x;\n",
        );
        full_code.push_str(
            "    let pT = textureLoad(pressure, (coords + vec3<i32>(0, 1, 0)) % dim, 0).x;\n",
        );
        full_code.push_str(
            "    let pF = textureLoad(pressure, (coords - vec3<i32>(0, 0, 1) + dim) % dim, 0).x;\n",
        );
        full_code.push_str(
            "    let pBK = textureLoad(pressure, (coords + vec3<i32>(0, 0, 1)) % dim, 0).x;\n",
        );
    } else {
        full_code.push_str("    let pL = textureLoad(pressure, clamp(coords - vec3<i32>(1, 0, 0), vec3<i32>(0), dim - 1), 0).x;\n");
        full_code.push_str("    let pR = textureLoad(pressure, clamp(coords + vec3<i32>(1, 0, 0), vec3<i32>(0), dim - 1), 0).x;\n");
        full_code.push_str("    let pB = textureLoad(pressure, clamp(coords - vec3<i32>(0, 1, 0), vec3<i32>(0), dim - 1), 0).x;\n");
        full_code.push_str("    let pT = textureLoad(pressure, clamp(coords + vec3<i32>(0, 1, 0), vec3<i32>(0), dim - 1), 0).x;\n");
        full_code.push_str("    let pF = textureLoad(pressure, clamp(coords - vec3<i32>(0, 0, 1), vec3<i32>(0), dim - 1), 0).x;\n");
        full_code.push_str("    let pBK = textureLoad(pressure, clamp(coords + vec3<i32>(0, 0, 1), vec3<i32>(0), dim - 1), 0).x;\n");
    }

    full_code.push_str("    let vel = textureLoad(velocity, global_id, 0).xyz;\n");
    full_code.push_str(
        "    let grad = vec3<f32>(pR - pL, pT - pB, pBK - pF) * _params.half_inverse_cell_size;\n",
    );
    full_code.push_str("    textureStore(output, global_id, vec4<f32>(vel - grad, 1.0));\n");
    full_code.push_str("}\n");
}
