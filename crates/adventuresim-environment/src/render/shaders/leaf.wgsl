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
#import adventuresim_environment::forest::{Species, Leaf, linear}

#import adventuresim_environment::leaf_geometry::leaf_vertex
#import adventuresim_environment::leaf_surface::{leaf_shape, coverage}

// x: blades a cluster.
@group(3) @binding(0) var<uniform> lod: vec4<u32>;
@group(3) @binding(1) var<storage, read> leaves: array<Leaf>;
@group(3) @binding(2) var<storage, read> species: array<Species>;
#ifdef FOREST_SHARED_TREES
#import adventuresim_environment::tree_pose::{TreeInstance, TreeVertex, place_tree, tree_normal, tree_visible}
@group(3) @binding(4) var<storage, read> placements: array<TreeInstance>;
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

@vertex
#ifdef FOREST_SHARED_TREES
fn vertex(vertex: TreeVertex) -> LeafOut {
#else
fn vertex(vertex: Vertex) -> LeafOut {
#endif
    let blades = lod.x;
#ifdef FOREST_SHARED_TREES
    let placement = placements[vertex.i_index];
    let address = vertex.address;
    let record = address / (blades * 4u);
    if record >= placement.records.w
        || !tree_visible(placement, view.lod_view_world_position.xyz) {
        var clipped: LeafOut;
        clipped.clip = vec4<f32>(0.0, 0.0, -1.0, 1.0);
        return clipped;
    }
    let inst = leaves[placement.records.z + record];
    let i = (address / 4u) % blades;
    let corner = vec2<f32>(f32(address % 2u), f32((address / 2u) % 2u));
#else
    let inst = leaves[vertex.i_index];
    let i = u32(round(vertex.position.z));
    let corner = vec2<f32>(vertex.position.x, vertex.position.y);
#endif
    let geometry = leaf_vertex(inst, species[inst.kind], i, corner, blades);
    var world = geometry.position;
    var normal = geometry.normal;
    var out: LeafOut;
    out.seed = geometry.seed;
    out.surface = world;
#ifdef FOREST_SHARED_TREES
    let pose = placement.pose;
    world = place_tree(pose, world);
    normal = tree_normal(pose, normal);
#endif
    out.world = vec4<f32>(world, 1.0);
    out.clip = view.clip_from_world * out.world;
    out.normal = normal;
    out.kind = inst.kind;
    out.light = inst.axis_light.w;
    out.uv = geometry.uv;
    out.crown = inst.crown;
    return out;
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
    pbr_input.material.flags |= pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;
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
