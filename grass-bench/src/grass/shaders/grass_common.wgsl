// Shared grass vertex math and fragment shading, ported from Fabelgeist's
// tactical_grass_instanced.wgsl. The two entry shaders (grass_eidolon.wgsl,
// grass_simple.wgsl) differ only in where the per-instance/batch data and the
// material uniform are bound.
//
// Mesh vertex contract:
//   uv    = (side 0|1, height fraction; tip uses 0.5)
//   uv_b  = blade root offset in tuft-local xz
//   color = species pigment, alpha = per-blade threshold/variation hash
// Per-instance data (locations 8-12, bevy_eidolon's layout): root position +
// scale, yaw, index, batch id, and a seed whose low byte is the ground-cover
// coverage sampled at placement time.

#define_import_path bench::grass_common

#import bevy_pbr::{
    mesh_view_bindings::{globals, lights, view},
    shadows,
}

struct GrassParams {
    // Wind direction xy, strength, and time scale.
    wind: vec4<f32>,
    // Interactor world position (xyz) and interaction radius.
    interaction: vec4<f32>,
    // Interactor smoothed velocity (xyz) and push strength.
    interaction_motion: vec4<f32>,
    // Root occlusion, dryness lane, authored lean, width compensation.
    params: vec4<f32>,
    // y scales the flat ambient term; x/z/w reserved.
    shading: vec4<f32>,
}

struct GrassVertex {
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
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
    @location(8) i_pos_scale: vec4<f32>,
    @location(9) i_rotation: f32,
    @location(10) i_index: u32,
    @location(11) i_batch_id: u32,
    @location(12) i_seed: u32,
}

struct GrassVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) visibility_range_dither: i32,
    @location(1) world_position: vec4<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
}

// Instance yaw + uniform scale + translation under the batch transform
// (bevy_eidolon::render::utils::calc_instance_world_matrix).
fn calc_instance_world_matrix(
    i_pos_scale: vec4<f32>,
    i_rotation: f32,
    parent_transform: mat4x4<f32>,
) -> mat4x4<f32> {
    let scale = i_pos_scale.w;
    let translation = i_pos_scale.xyz;
    let c = cos(i_rotation) * scale;
    let s = sin(i_rotation) * scale;
    let instance_local = mat4x4<f32>(
        vec4<f32>(c, 0.0, -s, 0.0),
        vec4<f32>(0.0, scale, 0.0, 0.0),
        vec4<f32>(s, 0.0, c, 0.0),
        vec4<f32>(translation, 1.0),
    );
    return parent_transform * instance_local;
}

// Per-instance dither level from the tuft root's camera distance:
//
//     `lod_range.`          x        y        z        w           camera distance
//                   <-------+--------+--------+--------+-------->
//     Dither Level  -16    -16       0        0        16      16  Dither Level
fn visibility_range_dither_level(lod_range: vec4<f32>, camera_distance: f32) -> i32 {
    let offset = select(-16, 0, camera_distance >= lod_range.z);
    let bounds = select(lod_range.xy, lod_range.zw, camera_distance >= lod_range.z);
    let level = i32(round((camera_distance - bounds.x) / (bounds.y - bounds.x) * 16.0));
    return offset + clamp(level, 0, 16);
}

fn grass_vertex(
    vertex: GrassVertex,
    instance_matrix: mat4x4<f32>,
    batch_color: vec4<f32>,
    visibility_range: vec4<f32>,
    grass: GrassParams,
) -> GrassVertexOutput {
    var out: GrassVertexOutput;

    var vertex_normal = vec3<f32>(0.0, 0.0, 1.0);
#ifdef VERTEX_NORMALS
    vertex_normal = vertex.normal;
#endif
    var vertex_uv = vec2<f32>(0.5, 0.0);
#ifdef VERTEX_UVS_A
    vertex_uv = vertex.uv;
#endif
    var blade_root_local = vec3<f32>(0.0, 0.0, 0.0);
#ifdef VERTEX_UVS_B
    blade_root_local = vec3<f32>(vertex.uv_b.x, 0.0, vertex.uv_b.y);
#endif
    var blade_threshold = 0.0;
#ifdef VERTEX_COLORS
    blade_threshold = vertex.color.a;
#endif

    let root_world = (instance_matrix * vec4<f32>(blade_root_local, 1.0)).xyz;
    let spatial_noise = sin(root_world.x * 0.071 + root_world.z * 0.113)
        * sin(root_world.x * 0.037 - root_world.z * 0.053);

    // Placement-time ground coverage, packed into the instance seed's low
    // byte. Distance opening reproduces the meadow clump break-up.
    let ground_coverage = f32(vertex.i_seed & 0xffu) / 255.0;
    let meadow_zone = smoothstep(-0.36, 0.52, spatial_noise);
    let camera_distance = distance(root_world, view.world_position.xyz);
    let distance_opening = smoothstep(7.0, 24.0, camera_distance);
    let clump_coverage = mix(1.0, mix(0.54, 1.0, meadow_zone), distance_opening);
    let effective_coverage = ground_coverage * clump_coverage;
    let blade_visibility = select(1.0, 0.0, blade_threshold > effective_coverage);

    let bend = clamp(vertex_uv.y, 0.0, 1.0);
    let wind_direction = normalize(grass.wind.xy);
    let wind_cross = vec2<f32>(-wind_direction.y, wind_direction.x);

    let blade_variation = fract(
        blade_threshold * 1.73 + spatial_noise * 0.31
            + root_world.x * 0.013 - root_world.z * 0.017
    );
    let age_signal = fract(blade_threshold * 2.37 + 0.13);
    let mature_age = smoothstep(0.68, 0.94, age_signal);
    let juvenile_vigor = 0.28 + 0.78 * blade_variation * blade_variation;
    let mature_vigor = 0.48 + 0.58 * blade_variation;
    let edge_growth = mix(0.26, 1.0, smoothstep(0.04, 0.92, ground_coverage));
    let blade_vigor = mix(juvenile_vigor, mature_vigor, meadow_zone)
        * edge_growth
        * mix(1.0, 0.94, mature_age);

    let wave_position = dot(root_world.xz, wind_direction) * 0.22;
    let wind_time = globals.time * grass.wind.w;
    let gust = 0.68 + 0.32 * sin(wind_time * 0.29 + spatial_noise * 3.7);
    let primary_wave = sin(wind_time + wave_position + spatial_noise * 1.9);
    let flutter = sin(wind_time * 2.73 - wave_position * 1.61 + spatial_noise * 5.3);
    let lean_variation = fract(blade_variation * 1.618 + blade_threshold * 0.73);
    let lean_direction = normalize(vec2<f32>(
        fract(blade_threshold * 1.37 + 0.17) - 0.5,
        fract(blade_threshold * 2.11 + 0.63) - 0.5,
    ) + vec2<f32>(0.0001, 0.0));
    let lean_amount = 0.025 + 0.030 * lean_variation;
    let natural_lean = lean_direction * (lean_amount + 0.012 * mature_age);
    let wind_offset = (
        wind_direction * primary_wave * gust
        + wind_cross * flutter * 0.18
    ) * grass.wind.z;

    var interaction_offset = vec2<f32>(0.0, 0.0);
    var interaction_droop = 0.0;
    if grass.interaction.w > 0.0 {
        let from_player = root_world.xz - grass.interaction.xz;
        let player_distance = length(from_player);
        let velocity_xz = grass.interaction_motion.xz;
        let fallback_direction = normalize(velocity_xz + vec2<f32>(0.0001, 0.0));
        let push_direction = select(
            fallback_direction,
            normalize(from_player),
            player_distance > 0.001,
        );
        let player_push = (1.0 - smoothstep(0.18, grass.interaction.w, player_distance))
            * grass.interaction_motion.w;
        let motion_direction = normalize(velocity_xz + push_direction * 0.15);
        interaction_offset = (
            push_direction * 0.62 + motion_direction * min(length(velocity_xz) * 0.035, 0.28)
        ) * player_push;
        interaction_droop = player_push * 0.34;
    }

    let original_world_normal =
        normalize((instance_matrix * vec4<f32>(vertex_normal, 0.0)).xyz);

    // Single-ribbon reconstruction: rows follow one cubic curve; the ribbon
    // rotates toward the camera only as it becomes edge-on.
    let t = bend;
    let one_minus_t = 1.0 - t;
    let curve_profile = 3.0 * one_minus_t * one_minus_t * t * 0.06
        + 3.0 * one_minus_t * t * t * 0.5
        + t * t * t;
    let curve_derivative = 3.0 * one_minus_t * one_minus_t * 0.06
        + 6.0 * one_minus_t * t * (0.5 - 0.06)
        + 3.0 * t * t * (1.0 - 0.5);
    let authored_facing = normalize(original_world_normal.xz + vec2<f32>(0.0001, 0.0));
    let authored_amount = 0.65 + 0.30 * fract(blade_threshold * 1.91 + blade_variation * 0.47);
    let total_curve = authored_facing * grass.params.z * authored_amount
        + natural_lean
        + wind_offset
        + interaction_offset;

    let curve_offset = total_curve * curve_profile * blade_visibility;
    let world_up = normalize((instance_matrix * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz);
    let tangent = normalize(vec3<f32>(
        world_up.x * blade_vigor + total_curve.x * curve_derivative,
        world_up.y * blade_vigor - interaction_droop * 2.0 * t,
        world_up.z * blade_vigor + total_curve.y * curve_derivative,
    ));

    let collapsed_position = blade_root_local
        + (vertex.position - blade_root_local) * blade_visibility;
    let centre_local = vec3<f32>(
        blade_root_local.x,
        collapsed_position.y * blade_vigor,
        blade_root_local.z,
    );
    let centre_world = instance_matrix * vec4<f32>(centre_local, 1.0);

    let local_side = normalize(vec2<f32>(-vertex_normal.z, vertex_normal.x));
    let transformed_side =
        (instance_matrix * vec4<f32>(local_side.x, 0.0, local_side.y, 0.0)).xz;
    let side_scale = length(transformed_side);
    let original_side = normalize(transformed_side + vec2<f32>(0.0001, 0.0));
    let to_camera = normalize(view.world_position.xz - root_world.xz
        + vec2<f32>(0.0001, 0.0));
    var camera_side = vec2<f32>(-to_camera.y, to_camera.x);
    camera_side = select(camera_side, -camera_side, dot(camera_side, original_side) < 0.0);
    let edge_on = 1.0 - smoothstep(
        0.08,
        0.38,
        abs(dot(original_world_normal.xz, to_camera)),
    );
    let visible_side = normalize(mix(original_side, camera_side, edge_on * 0.88));
    let half_width = length(collapsed_position.xz - blade_root_local.xz)
        * side_scale
        * max(grass.params.w, 1.0);
    let signed_side = select(-1.0, 1.0, vertex_uv.x >= 0.5);
    let side_offset = visible_side * half_width * signed_side;
    var world_position = vec4<f32>(
        centre_world.x + curve_offset.x + side_offset.x,
        centre_world.y - interaction_droop * t * t * blade_visibility,
        centre_world.z + curve_offset.y + side_offset.y,
        centre_world.w,
    );

    // Instances outside the tier's band collapse to their root (zero scale):
    // the whole-chunk culling on the simple path is padded, and eidolon's
    // compute cull has already dropped them.
    let in_band = camera_distance >= visibility_range.x
        && camera_distance < visibility_range.w;
    world_position = select(vec4<f32>(root_world, 1.0), world_position, in_band);

    let shaped_world_normal = normalize(cross(
        tangent,
        vec3<f32>(visible_side.x, 0.0, visible_side.y),
    ));

    out.world_position = world_position;
    out.clip_position = view.clip_from_world * world_position;
    // Blade normals are biased toward the sky (the game's shading.z = 0.76).
    out.world_normal = normalize(mix(
        shaped_world_normal,
        vec3<f32>(0.0, 1.0, 0.0),
        0.76,
    ));
    out.uv = vertex_uv;
#ifdef VERTEX_COLORS
    out.color = vec4<f32>(vertex.color.rgb, 1.0) * batch_color;
#else
    out.color = batch_color;
#endif
    out.visibility_range_dither = visibility_range_dither_level(visibility_range, camera_distance);
    return out;
}

fn grass_fragment(
    in: GrassVertexOutput,
    is_front: bool,
    grass: GrassParams,
) -> vec4<f32> {
    // Complementary per-pixel partition between crossfading tiers. Adjacent
    // tiers share their fade band endpoints, so their dither levels quantize
    // complementarily; keeping the outgoing tier on `hash >= t` and the
    // incoming tier on the complement hands every pixel to exactly one tier -
    // no gap, no double-shaded overlap.
    if in.visibility_range_dither != 0 {
        let magnitude = clamp(f32(abs(in.visibility_range_dither)) / 16.0, 0.0, 1.0);
        let pixel_hash = fract(
            52.9829189
                * fract(dot(floor(in.clip_position.xy), vec2<f32>(0.06711056, 0.00583715))),
        );
        if in.visibility_range_dither > 0 {
            // Fading out with distance: keep the far side of the hash.
            if pixel_hash < magnitude {
                discard;
            }
        } else if pixel_hash >= 1.0 - magnitude {
            // Fading in: keep exactly the pixels the outgoing tier dropped.
            discard;
        }
    }
    let lod_coverage = 1.0;
    let height_fraction = clamp(in.uv.y, 0.0, 1.0);
    let root_self_shadow = mix(grass.params.x, 1.0, pow(height_fraction, 0.72));
    let centre_distance = abs(in.uv.x - 0.5) * 2.0;
    let centre_rib = mix(0.84, 1.0, smoothstep(0.12, 0.72, centre_distance));
    var base_normal = select(-in.world_normal, in.world_normal, is_front);
    // Keep both thin faces in the upper hemisphere.
    base_normal.y = abs(base_normal.y);
    base_normal = normalize(base_normal);

    // Fast foliage model: flat ambient + wrapped Lambert translucency + one
    // clamped cascade fetch. No image-based lighting, transmission or specular.
    let occlusion = root_self_shadow * centre_rib;
    let albedo = in.color.rgb;
    var lit = albedo * lights.ambient_color.rgb * grass.shading.y * occlusion;
    let view_z = dot(vec4<f32>(
        view.view_from_world[0].z,
        view.view_from_world[1].z,
        view.view_from_world[2].z,
        view.view_from_world[3].z,
    ), in.world_position);
    for (var light_index = 0u; light_index < lights.n_directional_lights;
        light_index += 1u)
    {
        let light = lights.directional_lights[light_index];
        let alignment = dot(base_normal, light.direction_to_light);
        // Wrapped diffuse: 64% reflects off the lit face, 36% passes through.
        let wrapped = saturate(alignment) * 0.64 + saturate(-alignment) * 0.36;
        let shadow = clamp(
            shadows::fetch_directional_shadow(
                light_index,
                in.world_position,
                base_normal,
                view_z,
                in.clip_position.xy,
            ),
            0.12,
            1.0,
        );
        // 1/pi: Lambert BRDF normalisation.
        lit += albedo * light.color.rgb * (wrapped * shadow * occlusion * 0.3183099);
    }
    return vec4<f32>(lit * view.exposure, lod_coverage);
}
