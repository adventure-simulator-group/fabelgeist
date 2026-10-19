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
    Species, PlantInfo, Branch, linear
}

#import adventuresim_environment::branch_geometry::branch_vertex
#import adventuresim_environment::branch_surface::{bark_coordinates, bark}

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
    let geometry = branch_vertex(branch, k, j, sides, spans);
    let surface = geometry.position;
    var world = vec4<f32>(surface, 1.0);
    var normal = geometry.normal;
#ifdef FOREST_SHARED_TREES
    let pose = placement.pose;
    world = vec4<f32>(place_tree(pose, surface), 1.0);
    normal = tree_normal(pose, normal);
#endif

    var out: BranchOut;
    out.world = world;
    out.clip = view.clip_from_world * world;
    out.normal = normal;
    out.surface = surface;
    out.light = geometry.light;
    out.segment = index;
    out.kind = plants[branch.plant].kind;
    return out;
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
