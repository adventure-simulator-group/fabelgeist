// Branch segments as instanced tubes. The mesh is a grid of rings: a vertex's
// position holds its column `k` around and its ring `j` along. Each instance
// is one segment, read by its index from `branches`; the vertex shader places
// the grid along the segment's axis, the cubic Hermite curve from ring to
// ring.
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::pbr_types
#import bevy_pbr::pbr_functions
#import bevy_pbr::mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT
#import bevy_eidolon::render::io_types::Vertex
#import adventuresim_environment::forest::{
    TAU, Species, PlantInfo, Branch, basis, octahedral, linear, bark_noise
}

// x: sides around a tube; y: spans along it.
@group(3) @binding(0) var<uniform> lod: vec4<u32>;
@group(3) @binding(1) var<storage, read> branches: array<Branch>;
@group(3) @binding(2) var<storage, read> species: array<Species>;
@group(3) @binding(3) var<storage, read> plants: array<PlantInfo>;
#ifdef FOREST_SHARED_TREES
#import adventuresim_environment::tree_pose::{TreeInstance, TreeVertex, place_tree, tree_normal, tree_visible}
@group(3) @binding(4) var<storage, read> placements: array<TreeInstance>;
#endif

struct BranchOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) light: f32,
    @location(3) @interpolate(flat) segment: u32,
    @location(4) @interpolate(flat) kind: u32,
    @location(5) surface: vec3<f32>,
};

struct AxisPoint {
    center: vec3<f32>,
    tangent: vec3<f32>,
};

// A point of a segment's axis at `s` in `[0, 1]`: its centre and tangent.
fn axis_at(branch: Branch, s: f32) -> AxisPoint {
    let p0 = branch.a.xyz;
    let p1 = branch.b.xyz;
    let t0 = octahedral(branch.ta);
    let t1 = octahedral(branch.tb);
    let len = max(distance(p0, p1), 1e-6);
    let s2 = s * s;
    let s3 = s2 * s;
    var out: AxisPoint;
    out.center = (2.0 * s3 - 3.0 * s2 + 1.0) * p0 + (s3 - 2.0 * s2 + s) * len * t0
        + (3.0 * s2 - 2.0 * s3) * p1 + (s3 - s2) * len * t1;
    out.tangent = normalize((6.0 * s2 - 6.0 * s) * p0 + (3.0 * s2 - 4.0 * s + 1.0) * len * t0
        + (6.0 * s - 6.0 * s2) * p1 + (3.0 * s2 - 2.0 * s) * len * t1);
    return out;
}

@vertex
#ifdef FOREST_SHARED_TREES
fn vertex(vertex: TreeVertex) -> BranchOut {
#else
fn vertex(vertex: Vertex) -> BranchOut {
#endif
    let sides = lod.x;
    let spans = lod.y;
#ifdef FOREST_SHARED_TREES
    let placement = placements[vertex.i_index];
    let columns = sides + 1u;
    let record = vertex.address / (columns * (spans + 1u));
    // Every triangle belongs to one record. Padding is fully clipped before
    // any branch/species/plant reads or span-start arithmetic.
    if record >= placement.records.y
        || !tree_visible(placement, view.lod_view_world_position.xyz) {
        var clipped: BranchOut;
        clipped.clip = vec4<f32>(0.0, 0.0, -1.0, 1.0);
        return clipped;
    }
    let index = placement.records.x + record;
    let k = vertex.address % columns;
    let j = (vertex.address / columns) % (spans + 1u);
#else
    let index = vertex.i_index;
    let k = u32(round(vertex.position.x));
    let j = u32(round(vertex.position.y));
#endif
    let branch = branches[index];
    let s = f32(j) / f32(spans);
    // The end rings are taken as stored, so a neighbour sharing one builds
    // the very same vertices.
    var point: AxisPoint;
    if (j == 0u) {
        point.center = branch.a.xyz;
        point.tangent = octahedral(branch.ta);
    } else if (j == spans) {
        point.center = branch.b.xyz;
        point.tangent = octahedral(branch.tb);
    } else {
        point = axis_at(branch, s);
    }
    let radius = mix(branch.a.w, branch.b.w, s);
    let frame = basis(point.tangent);
    let theta = TAU * f32(k) / f32(sides);
    let n = cos(theta) * frame[0] + sin(theta) * frame[1];
    let surface = point.center + n * radius;
    var world = vec4<f32>(surface, 1.0);
    var normal = n;
#ifdef FOREST_SHARED_TREES
    let pose = placement.pose;
    world = vec4<f32>(place_tree(pose, surface), 1.0);
    normal = tree_normal(pose, n);
#endif

    var out: BranchOut;
    out.world = world;
    out.clip = view.clip_from_world * world;
    out.normal = normal;
    out.surface = surface;
    out.light = mix(branch.light_start, branch.light, s);
    out.segment = index;
    out.kind = plants[branch.plant].kind;
    return out;
}

// Where a point lies on its segment: `u` a turn around the axis, `v` along
// it in turns.
fn bark_coordinates(branch: Branch, x: vec3<f32>) -> vec2<f32> {
    let p0 = branch.a.xyz;
    let chord = branch.b.xyz - p0;
    let len = max(length(chord), 1e-6);
    var s = clamp(dot(x - p0, chord) / (len * len), 0.0, 1.0);
    var point = axis_at(branch, s);
    for (var i = 0; i < 2; i++) {
        s = clamp(s + dot(x - point.center, point.tangent) / len, 0.0, 1.0);
        point = axis_at(branch, s);
    }
    let d = x - point.center;
    let along = dot(d, point.tangent);
    let radial = d - point.tangent * along;
    let out_dir = radial / max(length(radial), 1e-9);
    let frame = basis(point.tangent);
    let u = fract(atan2(dot(out_dir, frame[1]), dot(out_dir, frame[0])) / TAU + 1.0);
    let v = mix(branch.v_start, branch.v_end, s + along / len);
    return vec2<f32>(u, v);
}

// Bark at `(u, v)`: the albedo's factor, and how open the surface is. Ridges
// run along the branch with dark furrows between; a palm's stem is ringed by
// the scars of fallen fronds.
fn bark(u: f32, v: f32, palm: bool) -> vec2<f32> {
    let wander = bark_noise(vec2<f32>(u * 6.0, v * 2.0), 6.0);
    var ridge = abs(fract(u * 8.0 + (wander - 0.5) * 1.2) - 0.5) * 2.0;
    var furrow_depth = 0.45;
    if (palm) {
        ridge = abs(fract(v * 3.0 + (wander - 0.5) * 0.3) - 0.5) * 2.0;
        ridge = mix(1.0, ridge, 0.6 + 0.4 * bark_noise(vec2<f32>(u * 5.0, v * 40.0), 5.0));
        furrow_depth = 0.75;
    }
    let grain = bark_noise(vec2<f32>(u * 32.0, v * 12.0), 32.0);
    let patches = bark_noise(vec2<f32>(u * 3.0, v * 1.2), 3.0);
    let furrow = smoothstep(0.0, 0.35, ridge) * (0.7 + 0.3 * grain);
    return vec2<f32>(mix(furrow_depth, 1.1, furrow) * (0.8 + 0.4 * patches), furrow);
}

@fragment
fn fragment(in: BranchOut) -> @location(0) vec4<f32> {
    let branch = branches[in.segment];
    let palm = u32(species[in.kind].leaf.w) == 2u;
    let place = bark_coordinates(branch, in.surface);
    let pattern = bark(place.x, place.y, palm);
    let albedo = linear(species[in.kind].bark.rgb) * pattern.x;
    // Deep in a crown, and in the furrows, less sky reaches the bark.
    let ao = mix(0.5, 1.0, in.light) * mix(0.7, 1.0, pattern.y);

    var pbr_input: pbr_types::PbrInput = pbr_types::pbr_input_new();
    pbr_input.material.flags |= pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;
    pbr_input.material.base_color = vec4<f32>(albedo, 1.0);
    pbr_input.material.perceptual_roughness = 0.9;
    pbr_input.material.metallic = 0.0;
    pbr_input.diffuse_occlusion = vec3<f32>(ao);
    pbr_input.frag_coord = in.clip;
    pbr_input.world_position = in.world;
    pbr_input.flags = MESH_FLAGS_SHADOW_RECEIVER_BIT;
    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr_input.world_normal = pbr_functions::prepare_world_normal(normalize(in.normal), false, false);
    pbr_input.N = normalize(pbr_input.world_normal);
    pbr_input.V = pbr_functions::calculate_view(in.world, pbr_input.is_orthographic);
    let lit = pbr_functions::apply_pbr_lighting(pbr_input);
    return pbr_functions::main_pass_post_lighting_processing(pbr_input, lit);
}
