#import bevy_pbr::{mesh_functions, view_transformations::position_world_to_clip}
#import adventuresim_environment::forest::{Species, PlantInfo, Branch, Leaf}
#import adventuresim_environment::grown_vertices::{LocalVertex, grown_wood, grown_leaf}
#import adventuresim_environment::branch_geometry::branch_light
#import adventuresim_environment::leaf_geometry::leaf_seed
#import adventuresim_environment::leaf_surface::{leaf_shape, coverage}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> shape: vec4<u32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<storage, read> species: array<Species>;
#ifdef GROWN_LEAF
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> leaves: array<Leaf>;
#else
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> branches: array<Branch>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<storage, read> plants: array<PlantInfo>;
#endif

struct GrownVertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) address: u32,
};

struct GrownVaryings {
    @builtin(position) position: vec4<f32>,
    @location(0) world: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) surface: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) @interpolate(flat) record: u32,
    @location(5) light: f32,
    @location(6) @interpolate(flat) seed: f32,
    @location(7) @interpolate(flat) kind: u32,
#ifdef MOTION_VECTOR_PREPASS
    @location(8) previous_world: vec4<f32>,
#endif
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    @location(9) unclipped_depth: f32,
#endif
};

@vertex
fn vertex(vertex: GrownVertex) -> GrownVaryings {
    var out: GrownVaryings;
    var local: LocalVertex;
#ifdef GROWN_LEAF
    let record = vertex.address / (shape.z * 4u);
    let leaf = leaves[record];
    let foliage = u32(species[leaf.kind].leaf.w);
    local = grown_leaf(leaf, foliage, vertex.address, shape, vertex.position, vertex.normal, vertex.uv);
    out.kind = leaf.kind;
    out.light = leaf.axis_light.w;
    out.seed = leaf_seed(leaf.seed, (vertex.address / 4u) % shape.z, foliage);
#else
    let record = vertex.address / ((shape.x + 1u) * (shape.y + 1u));
    let branch = branches[record];
    local = grown_wood(branch, vertex.address, shape, vertex.position, vertex.normal, vertex.uv);
    out.kind = plants[branch.plant].kind;
    let j = (vertex.address / (shape.x + 1u)) % (shape.y + 1u);
    out.light = branch_light(branch, f32(j) / f32(shape.y));
    out.seed = 0.0;
#endif
    // Keep allocator/builtin lookup at the entry, as in Bevy's own mesh shader.
    let model = mesh_functions::get_world_from_local(vertex.instance_index);
    let world = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(local.position, 1.0));
    out.world = world;
    out.position = position_world_to_clip(world.xyz);
    out.normal = mesh_functions::mesh_normal_local_to_world(local.normal, vertex.instance_index);
    out.surface = local.position;
    out.uv = local.uv;
    out.record = record;
#ifdef MOTION_VECTOR_PREPASS
    let previous_model = mesh_functions::get_previous_world_from_local(vertex.instance_index);
    out.previous_world = mesh_functions::mesh_position_local_to_world(previous_model, vec4<f32>(local.position, 1.0));
#endif
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.unclipped_depth = out.position.z;
    out.position.z = min(out.position.z, 1.0);
#endif
    return out;
}

#ifdef PREPASS_PIPELINE
#ifdef PREPASS_FRAGMENT
#import bevy_pbr::prepass_io::FragmentOutput
#ifdef MOTION_VECTOR_PREPASS
#import bevy_pbr::{mesh_view_bindings::view, prepass_bindings::previous_view_uniforms}
#endif
@fragment
fn fragment(in: GrownVaryings, @builtin(front_facing) front: bool) -> FragmentOutput {
#else
@fragment
fn fragment(in: GrownVaryings, @builtin(front_facing) front: bool) {
#endif
#ifdef GROWN_LEAF
    let alpha = coverage(leaf_shape(in.uv, u32(species[in.kind].leaf.w), in.seed));
    // Same analytic outline in color, camera depth/normal and shadow maps.
    // Depth and shadow are binary at this original source discard boundary.
    if alpha < 0.02 { discard; }
#endif
#ifdef PREPASS_FRAGMENT
    var out: FragmentOutput;
#ifdef NORMAL_PREPASS
    var normal = normalize(in.normal);
#ifdef GROWN_LEAF
    normal = select(-normal, normal, front);
#endif
    out.normal = vec4<f32>(normal * 0.5 + vec3<f32>(0.5), 1.0);
#endif
#ifdef MOTION_VECTOR_PREPASS
    let current_clip = view.unjittered_clip_from_world * in.world;
    let previous_clip = previous_view_uniforms.clip_from_world * in.previous_world;
    out.motion_vector = (current_clip.xy / current_clip.w - previous_clip.xy / previous_clip.w) * vec2<f32>(0.5, -0.5);
#endif
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.frag_depth = in.unclipped_depth;
#endif
    return out;
#endif
}
#else
#import adventuresim_environment::branch_surface::branch_response
#import adventuresim_environment::leaf_surface::leaf_response
#import adventuresim_environment::surface_lighting::light_surface
@fragment
fn fragment(in: GrownVaryings, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
#ifdef GROWN_LEAF
    let leaf = leaves[in.record];
    let alpha = coverage(leaf_shape(in.uv, u32(species[in.kind].leaf.w), in.seed));
    if alpha < 0.02 { discard; }
    let normal = select(-in.normal, in.normal, front);
    let response = leaf_response(species[in.kind], in.uv, in.seed, in.surface, leaf.crown, in.light);
    var color = light_surface(response, alpha, in.world, normal, in.position);
    color.a = alpha;
    return color;
#else
    let response = branch_response(branches[in.record], species[in.kind], in.surface, in.light);
    return light_surface(response, 1.0, in.world, normalize(in.normal), in.position);
#endif
}
#endif
