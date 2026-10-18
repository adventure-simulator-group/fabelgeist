// Leaf clusters as instanced blades. The mesh is a handful of quads: a
// vertex's position holds its corner across and along the blade and the
// blade's number. Each instance is the foliage of one shoot tip, read by its
// index from `leaves`: broadleaf blades set round the shoot in a golden-angle
// spiral and turned up to the sky, conifer needle sprays, or one arching palm
// frond. The outline is cut in the
// fragment shader.
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::pbr_types
#import bevy_pbr::pbr_functions
#import bevy_pbr::mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT
#import bevy_eidolon::render::io_types::Vertex
#import adventuresim_environment::forest::{TAU, Species, Leaf, basis, linear}

const GOLDEN: f32 = 2.39996323;

// x: blades a cluster.
@group(3) @binding(0) var<uniform> lod: vec4<u32>;
@group(3) @binding(1) var<storage, read> leaves: array<Leaf>;
@group(3) @binding(2) var<storage, read> species: array<Species>;
#ifdef FOREST_SHARED_TREES
#import adventuresim_environment::tree_pose::{TreePose, place_tree, tree_normal}
@group(3) @binding(4) var<storage, read> placements: array<TreePose>;
#endif

struct LeafOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec4<f32>,
    // The blade's upper side.
    @location(1) normal: vec3<f32>,
    @location(2) @interpolate(flat) kind: u32,
    @location(3) light: f32,
    // x across the blade, zero to one; y from its base to its tip.
    @location(4) uv: vec2<f32>,
    @location(5) @interpolate(flat) seed: f32,
    @location(6) @interpolate(flat) crown: vec4<f32>,
    @location(7) surface: vec3<f32>,
};

fn rand(seed: f32, i: u32, k: f32) -> f32 {
    return fract(sin(seed * 91.17 + f32(i) * 12.9898 + k * 78.233) * 43758.5453);
}

@vertex
fn vertex(vertex: Vertex) -> LeafOut {
    let blades = lod.x;
#ifdef FOREST_SHARED_TREES
    let address = u32(round(vertex.position.z));
    let inst = leaves[address / blades];
    let i = address % blades;
#else
    let inst = leaves[vertex.i_index];
    let i = u32(round(vertex.position.z));
#endif
    // Fewer blades cover as much, each larger.
    let spread = sqrt(10.0 / f32(blades));
    let corner = vec2<f32>(vertex.position.x, vertex.position.y);
    let foliage = u32(species[inst.kind].leaf.w);
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let tip = inst.center_size.xyz;
    let size = inst.center_size.w;
    let axis = normalize(inst.axis_light.xyz);
    let frame = basis(axis);
    let r0 = rand(inst.seed, i, 0.0);
    let r1 = rand(inst.seed, i, 1.0);
    let r2 = rand(inst.seed, i, 2.0);

    var out: LeafOut;
    var world: vec3<f32>;
    var normal: vec3<f32>;
    var uv = corner;
    if (foliage == 2u) {
        // One frond along `axis`, arching over under its own weight: blade
        // pairs along it, hinged at the rib.
        let side = select(-1.0, 1.0, i % 2u == 1u);
        let segments = f32(max(blades / 2u, 1u));
        let len = size * 4.0;
        let droop = 0.3 + 0.35 * fract(inst.seed * 13.7);
        let t = (f32(i / 2u) + corner.y) / segments;
        let along = normalize(axis - up * (2.0 * droop * t));
        var flat = cross(along, up);
        if (dot(flat, flat) < 1e-4) {
            flat = frame[0];
        }
        let level = normalize(flat);
        let face = normalize(cross(level, along)) * select(-1.0, 1.0, dot(cross(level, along), up) >= 0.0);
        let hinge = 0.5;
        let across = level * side;
        normal = normalize(face * cos(hinge) + across * sin(hinge));
        let spine = tip + (axis * t - up * droop * t * t) * len;
        let half = 0.5 * max(len * 0.3 * sin(3.14159 * min(t * 1.15, 1.0)), len * 0.02);
        let leaflet = across * cos(hinge) - face * sin(hinge);
        world = spine + leaflet * half * corner.x;
        uv = vec2<f32>(0.5 + 0.5 * side * corner.x, t);
        out.seed = inst.seed;
    } else {
        // Blades along the shoot's last stretch, the youngest at its tip.
        let needles = foliage == 1u;
        let t = (f32(i) + 0.7 * r0) / f32(blades);
        let attach = tip - axis * size * select(0.9, 1.6, needles) * t;
        let phi = inst.seed * TAU + f32(i) * GOLDEN;
        let radial = cos(phi) * frame[0] + sin(phi) * frame[1];
        let forward = select(0.55, 0.9, needles) + 0.3 * r1;
        let dir = normalize(radial + axis * forward - up * (0.15 + 0.35 * r2));
        var level = cross(up, dir);
        if (dot(level, level) < 1e-4) {
            level = frame[0];
        }
        level = normalize(level);
        let roll = (r0 - 0.5) * select(0.9, 1.4, needles);
        let above = cross(dir, level);
        let across = normalize(level * cos(roll) + above * sin(roll));
        normal = normalize(cross(dir, across));
        if (normal.y < 0.0) {
            normal = -normal;
        }
        var len = size * (0.62 + 0.3 * r1) * (1.0 - 0.25 * t) * spread;
        var width = len * (0.5 + 0.12 * r2);
        if (needles) {
            len = size * (1.3 + 0.4 * r1) * spread;
            width = len * 0.5;
        }
        world = attach + dir * len * uv.y + across * width * (uv.x - 0.5);
        out.seed = fract(inst.seed * 7.13 + f32(i) * 0.618);
    }

    out.surface = world;
#ifdef FOREST_SHARED_TREES
    let pose = placements[vertex.i_index];
    world = place_tree(pose, world);
    normal = tree_normal(pose, normal);
#endif
    out.world = vec4<f32>(world, 1.0);
    out.clip = view.clip_from_world * out.world;
    out.normal = normal;
    out.kind = inst.kind;
    out.light = inst.axis_light.w;
    out.uv = uv;
    out.crown = inst.crown;
    return out;
}

// The blade's outline, positive inside: an ovate toothed blade, a needle
// spray frayed at its edge, or a frond's leaflets.
fn leaf_shape(uv: vec2<f32>, foliage: u32, seed: f32) -> f32 {
    let x = abs(uv.x - 0.5);
    let y = uv.y;
    let b = clamp((y - 0.1) / 0.9, 0.0, 1.0);
    let half = 0.5 * pow(sin(3.14159 * pow(b, 0.7)), 0.85) * (1.0 - 0.15 * b);
    let teeth = 0.012 * abs(fract(b * 14.0 + seed) - 0.5);
    let blade = select(0.018 - x, half - teeth - x, y > 0.1);
    let broad = min(blade, min(y, 1.0 - y) * 0.5);
    let reach = 0.5 * pow(sin(3.14159 * min(y * 1.05, 1.0)), 0.4);
    let needle = abs(fract(y * 15.0 - x * 5.0 + seed * 3.0) - 0.5);
    let needles = min(reach - x, max(reach * 0.4 - x, (0.4 - needle) * 0.05));
    let leaflet = abs(fract(y * 22.0 - x * 4.0 + seed) - 0.5);
    let fronds = max(0.03 - x, min(0.48 - x, (0.3 - leaflet) * 0.06));
    if (foliage == 1u) {
        return needles;
    } else if (foliage == 2u) {
        return fronds;
    }
    return broad;
}

// How much of the pixel the outline covers: its distance over its rate of
// change across the pixel. Under multisampling it becomes sample coverage,
// so sub-pixel needles add up rather than shimmer.
fn coverage(sdf: f32) -> f32 {
    return clamp(sdf / max(fwidth(sdf), 1e-5) + 0.5, 0.0, 1.0);
}

@fragment
fn fragment(in: LeafOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let foliage = u32(species[in.kind].leaf.w);
    let alpha = coverage(leaf_shape(in.uv, foliage, in.seed));
    if (alpha < 0.02) {
        discard;
    }
    let n = select(-in.normal, in.normal, front);
    // Every blade its own shade of the species' green: some yellower, some
    // darker; the midrib paler.
    let r = in.seed;
    var albedo = linear(species[in.kind].leaf.rgb);
    albedo = mix(vec3<f32>(dot(albedo, vec3<f32>(0.3, 0.6, 0.1))), albedo, 0.8);
    albedo = mix(albedo, albedo * vec3<f32>(1.3, 1.1, 0.6), fract(r * 5.3) * 0.5);
    albedo *= 0.75 + 0.45 * fract(r * 11.7);
    let midrib = 1.0 - smoothstep(0.0, 0.025, abs(in.uv.x - 0.5));
    albedo = mix(albedo, albedo * vec3<f32>(1.6, 1.5, 1.1), midrib * 0.4);
    // Deep in a crown little sky gets through: the light the simulation
    // measured, and how far in the leaf sits.
    let depth = clamp(length(in.surface - in.crown.xyz) / max(in.crown.w, 1e-3), 0.0, 1.0);
    let ao = mix(0.3, 1.0, in.light) * mix(0.45, 1.0, smoothstep(0.1, 1.0, depth));

    var pbr_input: pbr_types::PbrInput = pbr_types::pbr_input_new();
    pbr_input.material.base_color = vec4<f32>(albedo, alpha);
    pbr_input.material.perceptual_roughness = 0.6;
    pbr_input.material.metallic = 0.0;
    pbr_input.diffuse_occlusion = vec3<f32>(ao);
    pbr_input.frag_coord = in.clip;
    pbr_input.world_position = in.world;
    pbr_input.flags = MESH_FLAGS_SHADOW_RECEIVER_BIT;
    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr_input.world_normal = pbr_functions::prepare_world_normal(n, false, false);
    pbr_input.N = normalize(pbr_input.world_normal);
    pbr_input.V = pbr_functions::calculate_view(in.world, pbr_input.is_orthographic);
    let lit = pbr_functions::apply_pbr_lighting(pbr_input);
    var color = pbr_functions::main_pass_post_lighting_processing(pbr_input, lit);
    // An opaque material's alpha is forced to one; the coverage is the point.
    color.a = alpha;
    return color;
}
