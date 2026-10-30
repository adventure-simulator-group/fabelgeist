// Shader ABI of the chunk-instanced renderer (`chunk_instancing.rs`).
//
// Every instanced material draws one mesh many times per chunk entity. Mesh
// attributes keep Bevy's locations; the per-instance stream follows at 8-10.
// Instances are already in world space, so the chunk entity's transform only
// steers Bevy's CPU culling and never reaches the shader.
#define_import_path fablegeist::chunk_instancing

#import bevy_pbr::mesh_view_bindings::view

struct Vertex {
    @location(0) position: vec3<f32>,
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_UVS_B
    @location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(4) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
    // World position (xyz) and uniform scale (w).
    @location(8) i_pos_scale: vec4<f32>,
    // Yaw about +Y, radians.
    @location(9) i_rotation: f32,
    // Content-defined bits; grass keeps placement coverage in the low byte.
    @location(10) i_seed: u32,
};

// The distance band a chunk's instances draw in: `xy` fade-in and `zw`
// fade-out, metres from the camera.
struct ChunkBand {
    visibility_range: vec4<f32>,
};

@group(2) @binding(0) var<uniform> chunk_band: ChunkBand;

fn instance_world_matrix(i_pos_scale: vec4<f32>, i_rotation: f32) -> mat4x4<f32> {
    let scale = i_pos_scale.w;
    let c = cos(i_rotation) * scale;
    let s = sin(i_rotation) * scale;
    return mat4x4<f32>(
        vec4<f32>(c, 0.0, -s, 0.0),
        vec4<f32>(0.0, scale, 0.0, 0.0),
        vec4<f32>(s, 0.0, c, 0.0),
        vec4<f32>(i_pos_scale.xyz, 1.0),
    );
}

// Bevy's visibility-range dither level for one instance, adapted from
// `bevy_pbr::mesh_functions`:
//
//     `lod_range.`          x        y        z        w           camera distance
//                   ←───────┼────────┼────────┼────────┼────────→
//     Dither Level  -16    -16       0        0        16      16  Dither Level
fn visibility_range_dither_level(lod_range: vec4<f32>, world_position: vec3<f32>) -> i32 {
    let camera_distance = length(view.world_position.xyz - world_position);
    let offset = select(-16, 0, camera_distance >= lod_range.z);
    let bounds = select(lod_range.xy, lod_range.zw, camera_distance >= lod_range.z);
    let level = i32(round((camera_distance - bounds.x) / (bounds.y - bounds.x) * 16.));
    return offset + clamp(level, 0, 16);
}
