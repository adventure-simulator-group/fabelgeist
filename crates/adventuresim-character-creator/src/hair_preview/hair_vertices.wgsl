// Hair's vertices, pulled from the buffers the GPU built them in.
//
// Hair is built on the GPU it is drawn on, into four buffers of four floats
// a vertex; its mesh carries only where each vertex is skinned, so no
// vertex comes back to the CPU. A vertex's index, less where its mesh starts
// in the shared vertex buffer, is its place in the hair's own buffers.

#define_import_path adventuresim::hair_vertices

#import bevy_pbr::mesh_bindings::mesh

// Position, and baked sky visibility.
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var<storage, read> hair_positions: array<vec4<f32>>;
// Normal, and the strip's own random value.
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var<storage, read> hair_normals: array<vec4<f32>>;
// Along the strands, root to tip.
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var<storage, read> hair_tangents: array<vec4<f32>>;
// Texture coordinates.
@group(#{MATERIAL_BIND_GROUP}) @binding(105) var<storage, read> hair_texcoords: array<vec4<f32>>;

// What a mesh would have carried: the random value as the second texture
// coordinates and the sky visibility as the colour, as hair's meshes did.
struct HairVertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    tangent: vec4<f32>,
    uv: vec2<f32>,
    uv_b: vec2<f32>,
    color: vec4<f32>,
}

fn hair_vertex(index: u32, instance_index: u32) -> HairVertex {
    let i = index - mesh[instance_index].first_vertex_index;
    let position = hair_positions[i];
    let normal = hair_normals[i];
    var vertex: HairVertex;
    vertex.position = position.xyz;
    vertex.normal = normal.xyz;
    vertex.tangent = vec4<f32>(hair_tangents[i].xyz, 1.0);
    vertex.uv = hair_texcoords[i].xy;
    vertex.uv_b = vec2<f32>(normal.w, 0.0);
    vertex.color = vec4<f32>(position.w, position.w, position.w, 1.0);
    return vertex;
}
