//! Authored face and all-face cube compute programs.

pub(super) fn face(shader_src: &str, storage_format: &str) -> String {
    format!(
        r#"
            {}

            @group(0) @binding(0) var out_cube: texture_storage_2d_array<{}, write>;

            struct Uniforms {{
                time: f32,
                face: u32,
                _pad1: f32,
                _pad2: f32,
            }};
            @group(0) @binding(1) var<uniform> uniforms: Uniforms;

            fn get_cube_direction(uv: vec2<f32>, face: u32) -> vec3<f32> {{
                let u = uv.x * 2.0 - 1.0;
                let v = uv.y * 2.0 - 1.0;
                var dir = vec3<f32>(0.0);
                if (face == 0u) {{ dir = vec3<f32>(1.0, -v, -u); }}
                else if (face == 1u) {{ dir = vec3<f32>(-1.0, -v, u); }}
                else if (face == 2u) {{ dir = vec3<f32>(u, 1.0, v); }}
                else if (face == 3u) {{ dir = vec3<f32>(u, -1.0, -v); }}
                else if (face == 4u) {{ dir = vec3<f32>(u, -v, 1.0); }}
                else {{ dir = vec3<f32>(-u, -v, -1.0); }}
                return normalize(dir);
            }}

            @compute @workgroup_size(8, 8, 1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
                let size = textureDimensions(out_cube).xy;
                if (id.x >= size.x || id.y >= size.y) {{
                    return;
                }}
                let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(size);
                let dir = get_cube_direction(uv, uniforms.face);
                let color = cube(dir);
                textureStore(out_cube, id.xy, uniforms.face, color);
            }}
            "#,
        shader_src, storage_format
    )
}

pub(super) fn all_faces(shader_src: &str, storage_format: &str) -> String {
    format!(
        r#"
            {}

            @group(0) @binding(0) var out_cube: texture_storage_2d_array<{}, write>;

            struct Uniforms {{
                time: f32,
                _pad0: f32,
                _pad1: f32,
                _pad2: f32,
            }};
            @group(0) @binding(1) var<uniform> uniforms: Uniforms;

            fn get_cube_direction(uv: vec2<f32>, face: u32) -> vec3<f32> {{
                let u = uv.x * 2.0 - 1.0;
                let v = uv.y * 2.0 - 1.0;
                var dir = vec3<f32>(0.0);
                if (face == 0u) {{ dir = vec3<f32>(1.0, -v, -u); }}
                else if (face == 1u) {{ dir = vec3<f32>(-1.0, -v, u); }}
                else if (face == 2u) {{ dir = vec3<f32>(u, 1.0, v); }}
                else if (face == 3u) {{ dir = vec3<f32>(u, -1.0, -v); }}
                else if (face == 4u) {{ dir = vec3<f32>(u, -v, 1.0); }}
                else {{ dir = vec3<f32>(-u, -v, -1.0); }}
                return normalize(dir);
            }}

            @compute @workgroup_size(8, 8, 1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
                let size = textureDimensions(out_cube).xy;
                if (id.x >= size.x || id.y >= size.y || id.z >= 6u) {{
                    return;
                }}
                let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(size);
                let dir = get_cube_direction(uv, id.z);
                let color = cube(dir);
                textureStore(out_cube, id.xy, id.z, color);
            }}
            "#,
        shader_src, storage_format
    )
}
