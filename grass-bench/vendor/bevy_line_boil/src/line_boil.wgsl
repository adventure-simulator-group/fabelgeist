// Line Boil Vertex Shader
// Creates a classic cartoon "line boil" effect via turbulent vertex displacement.
// Displacement is quantized to discrete frame intervals for that hand-drawn animation look.

#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    mesh_view_bindings::{globals, view, lights},
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    shadows::fetch_directional_shadow,
    skinning,
    morph::{morph_position, morph_normal, morph_tangent},
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    view_transformations::position_world_to_clip,
}

// Mirrors LineBoilSettings in lib.rs.
struct LineBoilSettings {
    intensity: f32,
    frame_rate: f32,
    noise_frequency: f32,
    seed: f32,
    fade_color: vec3<f32>,
    fade_in_secs: f32,
    fade_to_color_secs: f32,
    sweep_secs_per_meter: f32,
    _padding1: f32,
    _padding2: f32,
};

// Mirrors BoilShading in lib.rs.
struct BoilShading {
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    sun_dir: vec4<f32>,
    sun_color: vec4<f32>,
    sky_strength: f32,
    alpha_cutoff: f32,
    flags: u32,
    depth_bias: f32,
    fov_inv_tan: f32,
    screen_frames: f32,
    metallic: f32,
    roughness: f32,
    sheen: f32,
    dither_cell: f32,
    dither_freq: f32,
    dither_noise: f32,
    dither_height: f32,
    dither_shade: f32,
    _padding_a: f32,
    _padding_b: f32,
};

const BOIL_FLAG_LIT: u32 = 1u;
const BOIL_FLAG_ALPHA_MASK: u32 = 2u;
const BOIL_FLAG_GROUND_DITHER: u32 = 4u;

// Viewmodel screen sprite sheet: 8 columns, 19 rows, 60fps idle loop. Driven
// by the shader clock so the CPU never touches the material asset (touching
// it every frame re-created GPU buffers — the Firefox WebGPU churn pattern).
const SCREEN_COLS: i32 = 8;
const SCREEN_CELL: vec2<f32> = vec2<f32>(1.0 / 8.0, 1.0 / 19.0);
const SCREEN_FPS: f32 = 60.0;

// Depth slice the viewmodel is squeezed into, reverse-Z (1.0 = the near
// plane). The whole model lands in [0.99, 1.0], so nothing further than
// ~1.01x the near distance can poke through it, while its own parts keep
// sorting against each other. Cheaper than a second camera.
const VIEWMODEL_DEPTH_MIN: f32 = 0.99;

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> line_boil: LineBoilSettings;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> shading: BoilShading;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var base_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var base_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var sky_cube: texture_cube<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var sky_sampler: sampler;

// ============================================================================
// Smooth value noise functions (spatially coherent - nearby vertices move together)
// ============================================================================

// Simple hash function for 3D input -> single float
fn hash31(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 += dot(p3, p3.zyx + 31.32);
    return fract((p3.x + p3.y) * p3.z);
}

// Smooth interpolation
fn smooth_interp(t: f32) -> f32 {
    return t * t * (3.0 - 2.0 * t);
}

// 3D value noise with smooth trilinear interpolation
// This ensures nearby points get similar values (no vertex clipping)
fn value_noise_3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    // Smooth interpolation weights
    let u = vec3<f32>(smooth_interp(f.x), smooth_interp(f.y), smooth_interp(f.z));

    // Hash at 8 corners of the cell
    let n000 = hash31(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash31(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash31(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash31(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash31(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash31(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash31(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash31(i + vec3<f32>(1.0, 1.0, 1.0));

    // Trilinear interpolation
    let n00 = mix(n000, n100, u.x);
    let n10 = mix(n010, n110, u.x);
    let n01 = mix(n001, n101, u.x);
    let n11 = mix(n011, n111, u.x);
    let n0 = mix(n00, n10, u.y);
    let n1 = mix(n01, n11, u.y);
    return mix(n0, n1, u.z) * 2.0 - 1.0;  // Return -1 to 1
}

// Quantize time to create frame-held effect
fn quantize_time(time: f32, fps: f32) -> f32 {
    return floor(time * fps);
}

// Smooth 3D displacement vector - nearby vertices get similar displacement
fn smooth_turbulent_noise(pos: vec3<f32>, time_q: f32, seed: f32) -> vec3<f32> {
    let p = pos * line_boil.noise_frequency + seed;
    let t = time_q;

    // Sample smooth noise for each axis with different offsets
    // This creates a coherent wave-like displacement field
    return vec3<f32>(
        value_noise_3d(p + vec3<f32>(t * 1.0, 0.0, 0.0)),
        value_noise_3d(p + vec3<f32>(0.0, t * 1.3, 100.0)),
        value_noise_3d(p + vec3<f32>(200.0, 0.0, t * 0.7))
    );
}

// ============================================================================
// Vertex shader entry point
//
// A faithful copy of bevy 0.19's mesh.wgsl `vertex()` (the old pasted version
// predated it by two releases), plus the boil displacement at the end. Keep
// every `vertex_no_morph.instance_index`: reading `instance_index` through
// the `var vertex` copy hits a naga dx12 miscompile (gfx-rs/naga#2416) —
// on Firefox WebGPU (naga + dx12) that returned the WRONG instance index,
// so boil meshes rendered with other entities' transforms.
// ============================================================================

#ifdef MORPH_TARGETS
// The instance_index parameter must match vertex_in.instance_index. This is a work around for a wgpu dx12 bug.
// See https://github.com/gfx-rs/naga/issues/2416
fn morph_vertex(vertex_in: Vertex, instance_index: u32) -> Vertex {
    var vertex = vertex_in;
    let first_vertex = mesh[instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;

    let weight_count = bevy_pbr::morph::layer_count(instance_index);
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i, instance_index);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph_position(vertex_index, i, instance_index);
#ifdef VERTEX_NORMALS
        vertex.normal += weight * morph_normal(vertex_index, i, instance_index);
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent += vec4(weight * morph_tangent(vertex_index, i, instance_index), 0.0);
#endif
    }
    return vertex;
}
#endif

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph, vertex_no_morph.instance_index);
#else
    var vertex = vertex_no_morph;
#endif

    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);

#ifdef SKINNED
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var world_from_local = skinning::skin_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex_no_morph.instance_index
    );
#else
    var world_from_local = mesh_world_from_local;
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(world_from_local, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));

    // ========================================================================
    // Departures from stock mesh.wgsl: viewmodel FOV reprojection, line boil
    // displacement, depth bias.
    // ========================================================================
    var clip_position: vec4<f32>;
    if shading.fov_inv_tan > 0.0 {
        // Viewmodel: render at its own FOV without a second camera — scale
        // view-space XY before the camera's own projection.
        // `clip_from_view[1][1]` is `1/tan(world_fov/2)`, so dividing our
        // `1/tan(viewmodel_fov/2)` by it makes the camera's FOV cancel and the
        // viewmodel's substitute. Derived on the GPU so the animating
        // speed-FOV never dirties the material (rebinding all bind groups
        // every frame cost ~57ms/s on Firefox).
        let view_position = view.view_from_world * out.world_position;
        let view_scale = shading.fov_inv_tan / view.clip_from_view[1][1];
        clip_position = view.clip_from_view * vec4<f32>(view_position.xy * view_scale, view_position.zw);
        // Remap into the near-plane depth slice (see VIEWMODEL_DEPTH_MIN).
        // Scaled by w so it survives the perspective divide.
        let ndc_z = clip_position.z / clip_position.w;
        clip_position.z = (VIEWMODEL_DEPTH_MIN + ndc_z * (1.0 - VIEWMODEL_DEPTH_MIN)) * clip_position.w;
    } else {
        clip_position = position_world_to_clip(out.world_position.xyz);
    }

    // Quantize time to create frame-held effect (classic animation look)
    let time_quantized = quantize_time(globals.time, line_boil.frame_rate);

    // Use screen-space position (NDC) for noise - movement through 3D space won't affect boil
    let screen_pos = clip_position.xy / clip_position.w;
    let noise = smooth_turbulent_noise(vec3<f32>(screen_pos, 0.0), time_quantized, line_boil.seed);

    // Displace in screen space (X and Y only) - like lines drawn on paper wobbling
    // Scale by w to keep displacement consistent regardless of depth
    clip_position.x += noise.x * line_boil.intensity * clip_position.w;
    clip_position.y += noise.y * line_boil.intensity * clip_position.w;

    // Depth bias (anime hair overlay): positive pulls toward the camera in
    // reverse-Z NDC. Scaled by w to survive the perspective divide.
    clip_position.z += shading.depth_bias * clip_position.w;

    out.position = clip_position;
#endif

#ifdef VERTEX_UVS_A
    var uv = vertex.uv;
    // Viewmodel screen: scroll the sprite sheet on the shader clock.
    if shading.screen_frames > 0.5 {
        let frame = i32(floor(globals.time * SCREEN_FPS)) % i32(shading.screen_frames);
        uv += vec2<f32>(f32(frame % SCREEN_COLS), f32(frame / SCREEN_COLS)) * SCREEN_CELL;
    }
    out.uv = uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.instance_index = vertex_no_morph.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex_no_morph.instance_index, mesh_world_from_local[3]);
#endif

    return out;
}

// ============================================================================
// Fragment shader: flat color (or one sun + diffuse sky cubemap when LIT)
// plus a spawn-in bayer dither. Deliberately no PBR: the whole game shades
// through this one small shader so WebGL2 can compile it in milliseconds.
//
// MeshTag carries the entity's spawn time in milliseconds (of globals.time,
// which wraps hourly). Tag 0 = spawned long ago = fully visible, so untagged
// meshes render normally. The fade runs in two phases: pixels dither in from
// transparent to solid white, then dither from white to their real color.
// ============================================================================

const GLOBALS_WRAP_SECS: f32 = 3600.0;
// Max seconds a death tag may sit in the future before it reads as a wrap.
const DEATH_LEAD_MAX: f32 = 60.0;

fn bayer4(p: vec2<u32>) -> f32 {
    var m = array<u32, 16>(0u, 8u, 2u, 10u, 12u, 4u, 14u, 6u, 3u, 11u, 1u, 9u, 15u, 7u, 13u, 5u);
    return (f32(m[(p.y % 4u) * 4u + (p.x % 4u)]) + 0.5) / 16.0;
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    let spawn_ms = mesh_functions::get_tag(in.instance_index);
    var age = globals.time - f32(spawn_ms) / 1000.0;
    // A tag up to DEATH_LEAD_MAX in the future is a scheduled DEATH, not a
    // clock wrap: |age| is then the time remaining, and running the spawn
    // sequence on it plays the same dither mirrored — solid, flash, gone.
    // (Debris stamps break+lifetime.) Beyond the lead window it's a wrap.
    // Edge: a death tag whose countdown straddles the hourly wrap flashes in
    // instead of out for its last seconds — hourly, ~0.6s, rubble; accepted.
    if age < -DEATH_LEAD_MAX {
        age += GLOBALS_WRAP_SECS;
    }
    age = abs(age);

    // Sweep AFTER the wrap fix: this may push age negative, which just means
    // "not started yet" up there (a negative phase ratio discards everything).
    // The fade sweeps from the bottom of the SCREEN to the top (fragcoord y
    // grows downward), so `sweep_secs_per_meter` reads as secs/screen-height.
    let height = 1.0 - in.position.y / view.viewport.w;
    age -= height * line_boil.sweep_secs_per_meter;

    let fade_in = line_boil.fade_in_secs;
    let to_color = line_boil.fade_to_color_secs;

    // -1 = fade finished: never discards, never flashes the fade color.
    var dither = -1.0;
    if age < fade_in + to_color {
        dither = bayer4(vec2<u32>(in.position.xy));
        // Phase 1: existence dithers in from transparent.
        if age / fade_in < dither {
            discard;
        }
    }

    var color = shading.base_color;
#ifdef VERTEX_UVS_A
    color *= textureSample(base_texture, base_sampler, in.uv);
#endif
#ifdef VERTEX_COLORS
    color *= in.color;
#endif
    if (shading.flags & BOIL_FLAG_ALPHA_MASK) != 0u && color.a < shading.alpha_cutoff {
        discard;
    }

    // Retro ground texture: a chunky bayer dither picks between the base tone
    // and a shaded tone. The bayer grid is anchored to world XZ (not the
    // screen), so it sits on the floor and streams past when you move. The
    // threshold field is world value noise shifted by world height, so hills
    // and valleys read differently. Applied to albedo BEFORE lighting so the
    // sun still shades the pattern.
    if (shading.flags & BOIL_FLAG_GROUND_DITHER) != 0u {
        // +4096 keeps the f32->u32 cast off negative coords; bayer4 wraps %4
        // so the offset is invisible.
        let cell = vec2<u32>(floor(in.world_position.xz / max(shading.dither_cell, 0.01)) + 4096.0);
        let field = 0.5
            + value_noise_3d(in.world_position.xyz * shading.dither_freq) * shading.dither_noise
            + in.world_position.y * shading.dither_height;
        if field < bayer4(cell) {
            color = vec4(color.rgb * shading.dither_shade, color.a);
        }
    }

#ifdef VERTEX_NORMALS
    if (shading.flags & BOIL_FLAG_LIT) != 0u {
        var normal = normalize(in.world_normal);
        if !is_front {
            normal = -normal;
        }
        let view_vec = normalize(view.world_position - in.world_position.xyz);
        // Bevy's cascaded shadow map for the first directional light (the
        // game's sun, the same direction as shading.sun_dir). Only sampled
        // when the light has shadows on — the F9 "Shadows" switch — so the
        // shadowless path costs nothing.
        var shadow = 1.0;
        if lights.n_directional_lights > 0u
            && (lights.directional_lights[0].flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            let view_z = dot(vec4<f32>(
                view.view_from_world[0].z,
                view.view_from_world[1].z,
                view.view_from_world[2].z,
                view.view_from_world[3].z,
            ), in.world_position);
            shadow = fetch_directional_shadow(0u, in.world_position, normal, view_z, in.position.xy);
        }
        let sun = shading.sun_color.rgb * max(dot(normal, shading.sun_dir.xyz), 0.0) * shadow;
        let sky = textureSample(sky_cube, sky_sampler, normal).rgb * shading.sky_strength;
        var light = sun + sky;
        // Metal: swap the sky-diffuse fill for a view-dependent reflection of
        // the same cubemap. Diffuse-convolved, so it reads as soft gloss.
        if shading.metallic > 0.0 {
            // Roughness steers the sample between mirror (0) and normal (1) —
            // rough metal is only faintly view-dependent.
            let refl_dir = normalize(mix(reflect(-view_vec, normal), normal, shading.roughness));
            let refl = textureSample(sky_cube, sky_sampler, refl_dir).rgb * shading.sky_strength;
            light = mix(light, sun + refl, shading.metallic);
        }
        // Sheen, gated by the material's metallic factor: only surfaces
        // authored with metallic pick up the albedo-independent sky lift
        // (blacks toward sky color). Deliberately NOT what PBR does — there
        // every dielectric gets ~4% — but "shiny things shine, the world
        // stays clean" is the look v wants. Added AFTER the albedo multiply.
        color = vec4(
            color.rgb * light + sky * shading.sheen * shading.metallic + shading.emissive.rgb,
            color.a,
        );
    }
#endif

    var out: FragmentOutput;
    out.color = color;

    // Phase 2: surviving pixels dither from the flash color to their real
    // color. During phase 1 this ratio is negative, so everything flashes.
    if (age - fade_in) / to_color < dither {
        out.color = vec4(line_boil.fade_color, out.color.a);
    }
    return out;
}
