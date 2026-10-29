// Chunk-instanced distant vista tree cards.
//
// A port of the level-4 branch of `tactical_tree_impostor.wgsl` onto the
// chunk instance buffer. Vista stands are always whole-tree camera-facing
// cards, so the representation branch is gone; what remains is the billboard
// basis, the wind sway, the eight-view atlas selection and the cutout.
//
// The near cutoff is the one thing that could not move to the chunk. The
// per-entity renderer gave every card its own `VisibilityRange` whose start
// distance scales with that card's height, keeping a stand out of the near
// field until it subtends at most 16 degrees. Chunk bands are shared, so
// that test is evaluated here per instance instead, from the instance's own
// uniform scale.

diagnostic(off, derivative_uniformity);

#import bevy_pbr::{
    mesh_view_bindings::{globals, view},
    view_transformations::position_world_to_clip,
}

#import fablegeist::chunk_instancing::{
    Vertex, chunk_band, instance_world_matrix, visibility_range_dither_level,
}

@group(3) @binding(0) var baked_color: texture_2d<f32>;
@group(3) @binding(1) var baked_color_sampler: sampler;

struct TacticalVistaTreeInstancedMaterial {
    /// Unscaled card height in x, deterministic seed in y, wind strength in z,
    /// wind speed in w. The per-entity material carried the representation
    /// level in x; every vista card is level 4, so that slot now carries the
    /// height the near cutoff needs.
    parameters: vec4<f32>,
    lighting: vec4<f32>,
    ambient: vec4<f32>,
}

@group(3) @binding(2) var<uniform> tree: TacticalVistaTreeInstancedMaterial;

/// Half of the 16-degree maximum angular height, pre-tangented.
const VISTA_TREE_HALF_ANGLE_TANGENT: f32 = 0.14054084;

struct VistaCardVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
#ifdef VISIBILITY_RANGE_DITHER
    @location(0) @interpolate(flat) visibility_range_dither: i32,
#endif
    @location(1) world_position: vec4<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
    /// Distance at which this card is allowed to appear, from its own scale.
    @location(4) @interpolate(flat) near_cutoff: f32,
}

@vertex
fn vertex(vertex: Vertex) -> VistaCardVertexOutput {
    var out: VistaCardVertexOutput;
    let instance_matrix = instance_world_matrix(vertex.i_pos_scale, vertex.i_rotation);
    let root_world = (instance_matrix * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    let scale = vertex.i_pos_scale.w;

    // Whole-tree cards face the camera, so they cannot use the model basis.
    let to_camera = normalize(view.world_position.xz - root_world.xz + vec2<f32>(0.0001, 0.0));
    let right = vec3<f32>(to_camera.y, 0.0, -to_camera.x);
    var world_position = vec4<f32>(
        root_world
            + right * (vertex.position.x * scale)
            + vec3<f32>(0.0, vertex.position.y * scale, 0.0),
        1.0,
    );

    var uv = vec2<f32>(0.0, 0.0);
#ifdef VERTEX_UVS_A
    uv = vertex.uv;
#endif
    let height_weight = smoothstep(0.05, 1.0, uv.y);
    let phase = globals.time * tree.parameters.w
        + world_position.x * 0.11
        + world_position.z * 0.073
        + tree.parameters.y * 6.2831853;
    let wind = (sin(phase) * 0.72 + sin(phase * 2.17 + 1.3) * 0.28)
        * tree.parameters.z
        * height_weight
        * height_weight;
    world_position.x += wind;
    world_position.z += wind * 0.38;

    out.world_position = world_position;
    out.clip_position = position_world_to_clip(world_position.xyz);
    out.world_normal = normalize(mix(
        vec3<f32>(to_camera.x, 0.18, to_camera.y),
        vec3<f32>(0.0, 1.0, 0.0),
        0.22,
    ));
    out.uv = uv;
    // The scaled card must subtend no more than the maximum angular height.
    out.near_cutoff = (tree.parameters.x * scale) / (2.0 * VISTA_TREE_HALF_ANGLE_TANGENT);

#ifdef VISIBILITY_RANGE_DITHER
    let instance_origin = vec4<f32>(root_world, 1.0);
    out.visibility_range_dither =
        visibility_range_dither_level(chunk_band.visibility_range, instance_origin.xyz);
#endif
    return out;
}

@fragment
fn fragment(in: VistaCardVertexOutput) -> @location(0) vec4<f32> {
#ifdef VISIBILITY_RANGE_DITHER
    // Crisp handoff at the midpoint, as the per-entity path did: a translucent
    // crossfade reads as a pale duplicate crown.
    if in.visibility_range_dither <= -8 || in.visibility_range_dither > 8 {
        discard;
    }
#endif
    // Per-instance near cutoff. The per-entity renderer faded this over a 12%
    // band; keep that so a stand does not pop in as one hard edge.
    let camera_distance = distance(view.world_position.xz, in.world_position.xz);
    if camera_distance < in.near_cutoff {
        discard;
    }
    let near_fade = smoothstep(in.near_cutoff, in.near_cutoff * 1.12, camera_distance);
    let pixel_hash = fract(
        52.9829189 * fract(dot(floor(in.clip_position.xy), vec2<f32>(0.06711056, 0.00583715))),
    );
    if pixel_hash > near_fade {
        discard;
    }

    let direction = normalize(view.world_position.xz - in.world_position.xz + vec2<f32>(0.0001, 0.0));
    let angle = atan2(direction.y, direction.x);
    // Bake-card right axes produce view normals one quarter turn behind their
    // authored angle; select by that convention.
    let wrapped = fract(angle / 6.2831853 + 1.0 - 0.25);
    let view_index = u32(round(wrapped * 8.0)) % 8u;
    let column = view_index % 3u;
    let row = view_index / 3u;
    let uv = vec2<f32>(
        (f32(column) + in.uv.x) / 3.0,
        (f32(row) + in.uv.y) / 3.0,
    );

    let baked = textureSample(baked_color, baked_color_sampler, uv);
    if baked.a < 0.2 {
        discard;
    }
    let light_direction = normalize(tree.lighting.xyz);
    // The atlas already carries small-scale normal and occlusion variation;
    // apply only a broad symmetric wrap so an oblique card is not a black
    // cutout.
    let card_light = abs(dot(normalize(in.world_normal), light_direction));
    let normal_light = 0.25 + card_light * 0.75;
    let ambient_irradiance = tree.ambient.rgb * tree.ambient.w;
    let direct_irradiance = normal_light * tree.lighting.w * (1.0 - tree.ambient.w);
    return vec4<f32>(baked.rgb * (ambient_irradiance + vec3<f32>(direct_irradiance)), baked.a);
}
