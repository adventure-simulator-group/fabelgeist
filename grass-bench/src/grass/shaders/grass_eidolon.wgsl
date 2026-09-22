// bevy_eidolon entry points: batch transform, colour and fade band come from
// eidolon's per-batch storage buffer (group 2), the material from group 3.

#import bevy_eidolon::render::bindings::instance_uniforms
#import bench::grass_common::{
    GrassParams, GrassVertex, GrassVertexOutput,
    calc_instance_world_matrix, grass_vertex, grass_fragment,
}

@group(3) @binding(0) var<uniform> grass: GrassParams;

@vertex
fn vertex(vertex: GrassVertex) -> GrassVertexOutput {
    let batch = instance_uniforms[vertex.i_batch_id];
    let instance_matrix = calc_instance_world_matrix(
        vertex.i_pos_scale,
        vertex.i_rotation,
        batch.world_from_local,
    );
    return grass_vertex(vertex, instance_matrix, batch.color, batch.visibility_range, grass);
}

@fragment
fn fragment(
    in: GrassVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    return grass_fragment(in, is_front, grass);
}
