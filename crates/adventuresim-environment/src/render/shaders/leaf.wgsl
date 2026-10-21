// Leaf clusters as instanced blades. The mesh is a handful of quads: a
// vertex's position holds its corner across and along the blade and the
// blade's number. Each instance is the foliage of one shoot tip, read by its
// index from `leaves`: broadleaf blades set round the shoot in a golden-angle
// spiral and turned up to the sky, conifer needle sprays, or one arching palm
// frond. The outline is cut in the
// fragment shader.
#import bevy_pbr::mesh_view_bindings::view
#import bevy_eidolon::render::io_types::Vertex
#import adventuresim_environment::forest::{Species, Leaf, linear}

#import adventuresim_environment::leaf_geometry::leaf_vertex
#import adventuresim_environment::leaf_surface::{leaf_shape, coverage, leaf_response}

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
    // Surface normal, oriented to the visible side in the fragment.
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
    let geometry = leaf_vertex(inst, u32(species[inst.kind].leaf.w), i, corner, blades);
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

#import adventuresim_environment::surface_lighting::light_surface

@fragment
fn fragment(in: LeafOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let foliage = u32(species[in.kind].leaf.w);
    let alpha = coverage(leaf_shape(in.uv, foliage, in.seed));
    if (alpha < 0.02) {
        discard;
    }
    let n = select(-in.normal, in.normal, front);
    let response = leaf_response(species[in.kind], in.uv, in.seed, in.surface, in.crown, in.light);
    var color = light_surface(response, alpha, in.world, n, in.clip);
    // Preserve fractional multisample coverage after opaque postlighting.
    color.a = alpha;
    return color;
}
