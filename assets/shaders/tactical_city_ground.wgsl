// One production material for playable streets, distant streets, markets and yards.
// World metres anchor stone/grain scale. Local footprint coordinates describe
// wear and shoulders with constant work, never a scan of the city's patches.
#import bevy_pbr::{pbr_fragment::pbr_input_from_standard_material}
#ifdef PREPASS_PIPELINE
#import bevy_pbr::{prepass_io::{VertexOutput, FragmentOutput}, pbr_deferred_functions::deferred_output}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct CityGroundParameters {
    surface: vec4<f32>,
    weather: vec4<f32>,
    texture_scale: vec4<f32>,
}
@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> ground: CityGroundParameters;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var soil_height_ao: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var soil_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var stone_albedo: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var stone_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(105) var stone_arm: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(106) var stone_arm_sampler: sampler;

fn cell_hash(point: vec2<f32>) -> vec2<f32> {
    var p = fract(vec3<f32>(point.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p += dot(p, p.yzx + 33.33);
    return fract((p.xx + p.yz) * p.zy);
}

fn ground_noise(point: vec2<f32>) -> f32 {
    let cell = floor(point);
    let f = fract(point);
    let blend = f * f * (3.0 - 2.0 * f);
    return mix(mix(cell_hash(cell).x, cell_hash(cell + vec2<f32>(1.0, 0.0)).x, blend.x),
        mix(cell_hash(cell + vec2<f32>(0.0, 1.0)).x, cell_hash(cell + vec2<f32>(1.0)).x, blend.x), blend.y);
}

// Rounded irregular fieldstones: nearest-site cells establish a shared edge
// network, then a low crown rounds the exposed stone. No baked wall mortar.
fn fieldstones(point: vec2<f32>) -> vec3<f32> {
    let cell = floor(point);
    var nearest = 100.0;
    var next = 100.0;
    var variation = 0.5;
    for (var y = -1; y <= 1; y += 1) {
        for (var x = -1; x <= 1; x += 1) {
            let site = cell + vec2<f32>(f32(x), f32(y));
            let random = cell_hash(site);
            let centre = site + 0.5 + (random - 0.5) * 0.96;
            let delta = point - centre;
            let distance = dot(delta, delta);
            if distance < nearest {
                next = nearest;
                nearest = distance;
                variation = random.x;
            } else {
                next = min(next, distance);
            }
        }
    }
    let edge = max(sqrt(next) - sqrt(nearest), 0.0);
    // Rounded shoulders continue into uneven convex crowns instead of a
    // constant flat centre. Closely spaced sites produce smaller cobbles.
    let rounded = pow(max(1.0 - nearest * (1.15 + variation * 0.45), 0.0), 0.65);
    let crown = smoothstep(0.008, 0.15 + variation * 0.06, edge)
        * rounded * (0.68 + variation * 0.32);
    return vec3<f32>(crown, variation, edge);
}

fn height_normal(position: vec3<f32>, normal: vec3<f32>, height: f32) -> vec3<f32> {
    let dx = dpdx(position);
    let dy = dpdy(position);
    let rx = cross(dy, normal);
    let ry = cross(normal, dx);
    let determinant = dot(dx, rx);
    let denominator = select(-1.0, 1.0, determinant >= 0.0) * max(abs(determinant), 0.000001);
    return normalize(normal - (rx * dpdx(height) + ry * dpdy(height)) / denominator);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr = pbr_input_from_standard_material(in, is_front);
    let position = in.world_position.xyz;
    let world = position.xz;
    let footprint = in.color;
    let local = in.uv * footprint.xy;
    let half_width = footprint.x * 0.5;
    let across = local.x - half_width;
    let to_end = min(local.y, footprint.y - local.y);
    let to_side = min(local.x, footprint.x - local.x);
    let corridor = 1.0 - step(0.5, footprint.z);
    let market = 1.0 - abs(footprint.z - 1.0);
    let edge_distance = mix(min(to_side, to_end), to_side, corridor);
    let broad = ground_noise(world * 0.16);
    let broken = ground_noise(world * 1.7);
    let edge_width = 0.55 + broad * 0.95;
    let shoulder = 1.0 - smoothstep(0.12, edge_width, edge_distance + (broken - 0.5) * 0.65);

    // Fade wheel bands before intersections. A market owns broad pedestrian
    // crossing wear, while separate lower streets never show through it.
    let wheel_spacing = min(0.78, half_width * 0.45);
    let wander = (ground_noise(vec2<f32>(local.y * 0.1, 17.0)) - 0.5) * 0.18;
    let wheel = 1.0 - smoothstep(0.09, 0.31, abs(abs(across + wander) - wheel_spacing));
    let lane = (1.0 - smoothstep(0.4, half_width * 0.85, abs(across))) * 0.25;
    let approach = smoothstep(1.0, 4.5, to_end);
    let street_wear = max(wheel * (0.55 + broken * 0.45), lane) * approach;
    let plaza_crossing = max(
        1.0 - smoothstep(0.8, 2.8, abs(local.x - footprint.x * 0.5)),
        1.0 - smoothstep(0.8, 2.8, abs(local.y - footprint.y * 0.5)));
    let activity_wear = in.uv_b.x;
    let activity_dampness = in.uv_b.y;
    let wear = max(mix(plaza_crossing * market * (0.25 + broad * 0.35), street_wear, corridor),
        activity_wear * (0.75 + broken * 0.25));

    let soil = textureSample(soil_height_ao, soil_sampler, world * ground.texture_scale.x);
    let soil_height = dot(soil.rg, vec2<f32>(256.0 / 257.0, 1.0 / 257.0));
    let rock_color = textureSample(stone_albedo, stone_sampler, world * ground.texture_scale.y).rgb;
    let rock_arm = textureSample(stone_arm, stone_arm_sampler, world * ground.texture_scale.y).rgb;
    let stone_uv = world / ground.surface.y;
    let stones = fieldstones(stone_uv);
    // Average subpixel stones into the same stone/soil mixture. Keeping a
    // synthetic constant crown at distance would erase mud and alias bevels.
    let stone_pixel_span = max(length(dpdx(stone_uv)), length(dpdy(stone_uv)));
    let resolved = 1.0 - smoothstep(0.16, 0.7, stone_pixel_span);
    // Rock microtexture is a close-range grain source, not a repeated colour
    // tile across a whole square. Its averaged pigment/roughness retain the
    // broad soil deposits once individual cobbles become subpixel.
    let stone_color = mix(vec3<f32>(0.20), rock_color, resolved * 0.35);
    let arm = mix(vec3<f32>(0.96, 0.86, 0.0), rock_arm, resolved * 0.35);
    let crown = stones.x;
    let moisture = ground.weather.x;
    // Dirt fills stone joints before it covers the crowns. Traffic adds fine
    // soil and dampness but does not carve imaginary ruts through fieldstone.
    let deposit_noise = ground_noise(world * 0.65 + vec2<f32>(23.0, 71.0));
    let deposits = smoothstep(0.40, 0.73, deposit_noise * 0.65 + broad * 0.35);
    let soil_level = 0.075 + shoulder * 0.91 + deposits * (0.20 + wear * 0.8)
        + wear * 0.20 + activity_wear * deposits * 0.28;
    let edge_filter = max(fwidth(crown), 0.10);
    let exposed_near = smoothstep(soil_level - edge_filter, soil_level + edge_filter, crown);
    let exposed_average = 1.0 - smoothstep(0.02, 0.82, soil_level);
    let exposed_stone = mix(exposed_average, exposed_near, resolved) * ground.surface.x;
    let damp = clamp(moisture * (0.5 + 0.25 * wear + 0.25 * (1.0 - broad) + activity_dampness * 0.4), 0.0, 1.0);
    let earth_tint = mix(vec3<f32>(0.16, 0.117, 0.073), vec3<f32>(0.13, 0.105, 0.061), ground.surface.z);
    let earth = earth_tint * (0.78 + soil_height * 0.3 + broad * 0.18) * (1.0 - damp * 0.35);
    let stone_tone = mix(0.5, stones.y, resolved);
    let stone_tint = mix(vec3<f32>(0.64, 0.48, 0.30), vec3<f32>(0.94, 1.0, 1.07), stone_tone);
    let dust = deposits * (0.12 + wear * 0.22);
    let stones_tinted = mix(stone_color * stone_tint * (0.73 + stone_tone * 0.48),
        earth_tint * 1.35, dust) * (1.0 - damp * 0.2);
    var color = mix(earth, stones_tinted, exposed_stone);
    let snow = ground.weather.y;
    color = mix(color, vec3<f32>(0.79, 0.84, 0.86), snow);
    let macro_normal = normalize(cross(dpdy(position), dpdx(position)));
    let upward = select(-macro_normal, macro_normal, macro_normal.y >= 0.0);
    let earth_height = soil_height * 0.014 * (1.0 - wear * 0.55);
    // A buried stone emerges above its soil bed; the edge must not form an
    // artificial trench with a one-pixel dark rim. Fade unresolved relief
    // before its derivatives become a screen-space stipple pattern.
    let stone_height = earth_height + max(crown - soil_level, 0.0)
        * ground.surface.y * 0.065 * resolved;
    let composed_height = mix(earth_height, stone_height, exposed_stone) * (1.0 - snow * 0.8);
    let bump_resolved = 1.0 - smoothstep(0.08, 0.32, stone_pixel_span);
    pbr.N = normalize(mix(upward, height_normal(position, upward, composed_height), bump_resolved));
    pbr.world_normal = upward;
    pbr.material.base_color = vec4<f32>(color, 1.0);
    pbr.material.perceptual_roughness = mix(mix(0.96, arm.g, exposed_stone) - damp * 0.28, 0.92, snow);
    pbr.diffuse_occlusion = vec3<f32>(mix(mix(0.73, 1.0, soil.b), arm.r, exposed_stone));
#ifdef PREPASS_PIPELINE
    return deferred_output(in, pbr);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr);
    out.color = main_pass_post_lighting_processing(pbr, out.color);
    return out;
#endif
}
