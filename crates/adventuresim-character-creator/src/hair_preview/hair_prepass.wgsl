// Hair's vertex shader for the prepass and the shadow maps: Bevy's prepass
// vertex shader, its attributes pulled from the hair's own buffers rather
// than its mesh.

#import bevy_pbr::{
    mesh_functions,
    skinning,
    prepass_io::VertexOutput,
    view_transformations::position_world_to_clip,
}
#import adventuresim::hair_vertices::hair_vertex

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>,
#ifdef SKINNED
    @location(5) joint_indices: vec4<u32>,
    @location(6) joint_weights: vec4<f32>,
#endif
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let hair = hair_vertex(vertex.index, vertex.instance_index);
    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
#ifdef SKINNED
    let world_from_local = skinning::skin_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex.instance_index,
    );
#else
    let world_from_local = mesh_world_from_local;
#endif
    out.world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(hair.position, 1.0),
    );
    out.position = position_world_to_clip(out.world_position.xyz);
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.unclipped_depth = out.position.z;
    out.position.z = min(out.position.z, 1.0);
#endif
#ifdef VERTEX_UVS_A
    out.uv = hair.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = hair.uv_b;
#endif
#ifdef NORMAL_PREPASS_OR_DEFERRED_PREPASS
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(world_from_local, hair.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(hair.normal, vertex.instance_index);
#endif
#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        hair.tangent,
        vertex.instance_index,
    );
#endif
#endif
#ifdef VERTEX_COLORS
    out.color = hair.color;
#endif
#ifdef MOTION_VECTOR_PREPASS
#ifdef SKINNED
#ifdef HAS_PREVIOUS_SKIN
    let previous = skinning::skin_prev_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex.instance_index,
    );
#else
    let previous = mesh_functions::get_previous_world_from_local(vertex.instance_index);
#endif
#else
    let previous = mesh_functions::get_previous_world_from_local(vertex.instance_index);
#endif
    out.previous_world_position = mesh_functions::mesh_position_local_to_world(
        previous,
        vec4<f32>(hair.position, 1.0),
    );
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif
#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex.instance_index,
        mesh_world_from_local[3],
    );
#endif
    return out;
}
