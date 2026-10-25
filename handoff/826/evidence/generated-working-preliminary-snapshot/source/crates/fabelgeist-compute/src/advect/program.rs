//! Authored native fluid program; numerical operations are preserved.

use crate::prelude::*;

pub(super) fn source(
    mode: BoundaryMode,
    velocity_res: &ResourceDescriptor,
    quantity_res: &ResourceDescriptor,
    output_res: &ResourceDescriptor,
) -> Result<String> {
    let mut full_code = String::new();

    if !matches!(
        velocity_res,
        ResourceDescriptor::Texture2d(_) | ResourceDescriptor::Texture3d(_)
    ) {
        return Err(anyhow!("Advect only supports Texture2d or Texture3d"));
    }

    full_code.push_str(&velocity_res.to_wgsl_input_binding(0, 0, "velocity"));
    full_code.push_str(&quantity_res.to_wgsl_input_binding(0, 1, "quantity"));
    full_code.push_str(&output_res.to_wgsl_output_binding(0, 2, "output")?);
    full_code.push_str("\nstruct Parameters {\n");
    full_code.push_str("    size: vec4<f32>,\n");
    full_code.push_str("    dt: f32,\n");
    full_code.push_str("    _pad: vec3<f32>,\n");
    full_code.push_str("};\n");
    full_code.push_str("@group(0) @binding(3) var<uniform> _params: Parameters;\n\n");

    bilinear(&mut full_code, mode);

    match velocity_res {
        ResourceDescriptor::Texture2d(_) => plane(&mut full_code, mode),
        ResourceDescriptor::Texture3d(_) => volume(&mut full_code, mode),
        _ => return Err(anyhow!("Advect only supports Texture2d or Texture3d")),
    }
    Ok(full_code)
}

fn bilinear(full_code: &mut String, mode: BoundaryMode) {
    let wrap_logic = if mode == BoundaryMode::Wrap {
        "((pos % 1.0) + 1.0) % 1.0"
    } else {
        "clamp(pos, vec2<f32>(0.0), vec2<f32>(1.0))"
    };

    let bilinear_logic = format!(
        r#"
fn sample_bilinear(tex: texture_2d<f32>, pos: vec2<f32>) -> vec4<f32> {{
    let t_size = vec2<f32>(textureDimensions(tex));
    let wrapped_pos = {};
    let f_coords = wrapped_pos * t_size - 0.5;
    let i_coords = vec2<i32>(floor(f_coords));
    let frac = f_coords - vec2<f32>(i_coords);

    let t_size_i = vec2<i32>(t_size);
    "#,
        wrap_logic
    );

    full_code.push_str(&bilinear_logic);

    if mode == BoundaryMode::Wrap {
        full_code.push_str(
            r#"
    let c00 = textureLoad(tex, (i_coords + vec2<i32>(0, 0) + t_size_i) % t_size_i, 0);
    let c10 = textureLoad(tex, (i_coords + vec2<i32>(1, 0) + t_size_i) % t_size_i, 0);
    let c01 = textureLoad(tex, (i_coords + vec2<i32>(0, 1) + t_size_i) % t_size_i, 0);
    let c11 = textureLoad(tex, (i_coords + vec2<i32>(1, 1) + t_size_i) % t_size_i, 0);
"#,
        );
    } else {
        full_code.push_str(
            r#"
    let c00 = textureLoad(tex, clamp(i_coords + vec2<i32>(0, 0), vec2<i32>(0), t_size_i - 1), 0);
    let c10 = textureLoad(tex, clamp(i_coords + vec2<i32>(1, 0), vec2<i32>(0), t_size_i - 1), 0);
    let c01 = textureLoad(tex, clamp(i_coords + vec2<i32>(0, 1), vec2<i32>(0), t_size_i - 1), 0);
    let c11 = textureLoad(tex, clamp(i_coords + vec2<i32>(1, 1), vec2<i32>(0), t_size_i - 1), 0);
"#,
        );
    }

    full_code.push_str(
        r#"
    let r0 = mix(c00, c10, frac.x);
    let r1 = mix(c01, c11, frac.x);
    let res = mix(r0, r1, frac.y);

    return res;
}
"#,
    );
}

fn plane(full_code: &mut String, _mode: BoundaryMode) {
    full_code.push_str("@compute @workgroup_size(16, 16, 1)\n");
    full_code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
    full_code.push_str("    let tex_dim = textureDimensions(output);\n");
    full_code
        .push_str("    if (global_id.x >= tex_dim.x || global_id.y >= tex_dim.y) { return; }\n");
    full_code.push_str("    let coords = vec2<f32>(global_id.xy) + 0.5;\n");
    full_code.push_str("    let normalized_pos = coords / _params.size.xy;\n");
    full_code.push_str("    let vel_pixel = sample_bilinear(velocity, normalized_pos).xy;\n");
    full_code.push_str("    let vel_dim = vec2<f32>(textureDimensions(velocity));\n");
    full_code.push_str("    let vel_norm = vel_pixel / vel_dim;\n");
    full_code.push_str("    let pos = normalized_pos - vel_norm * _params.dt;\n");
    full_code.push_str("    let val = sample_bilinear(quantity, pos);\n");
    full_code.push_str("    textureStore(output, global_id.xy, val);\n");
    full_code.push_str("}\n");
}

fn volume(full_code: &mut String, mode: BoundaryMode) {
    full_code.push_str("@compute @workgroup_size(8, 8, 4)\n");
    full_code.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
    full_code.push_str("    let tex_dim = textureDimensions(output);\n");
    full_code.push_str("    if (global_id.x >= tex_dim.x || global_id.y >= tex_dim.y || global_id.z >= tex_dim.z) { return; }\n");
    full_code.push_str("    let coords = vec3<f32>(global_id) + 0.5;\n");
    full_code.push_str("    let normalized_pos = coords / _params.size.xyz;\n");
    full_code.push_str("    let vel_pixel = textureLoad(velocity, global_id, 0).xyz;\n");
    full_code.push_str("    let vel_dim = vec3<f32>(textureDimensions(velocity));\n");
    full_code.push_str("    let vel_norm = vel_pixel / vel_dim;\n");
    full_code.push_str("    let pos = normalized_pos - vel_norm * _params.dt;\n");

    if mode == BoundaryMode::Wrap {
        full_code.push_str("    let wrapped_pos = ((pos % 1.0) + 1.0) % 1.0;\n");
        full_code.push_str("    let val = textureLoad(quantity, vec3<u32>(wrapped_pos * vec3<f32>(textureDimensions(quantity))), 0);\n");
    } else {
        full_code.push_str("    let quantity_dimensions = textureDimensions(quantity);\n");
        full_code.push_str("    let quantity_coords = min(vec3<u32>(clamp(pos, vec3<f32>(0.0), vec3<f32>(1.0)) * vec3<f32>(quantity_dimensions)), quantity_dimensions - vec3<u32>(1));\n");
        full_code.push_str("    let val = textureLoad(quantity, quantity_coords, 0);\n");
    }
    full_code.push_str("    textureStore(output, global_id, val);\n");
    full_code.push_str("}\n");
}
