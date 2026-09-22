// Simple (chunked) entry points: instances are stored in world space, so the
// batch transform is the identity; the material and the tier's fade band are
// one uniform at group 2.

#import bench::grass_common::{
    GrassParams, GrassVertex, GrassVertexOutput,
    calc_instance_world_matrix, grass_vertex, grass_fragment,
}

struct GrassTierUniform {
    wind: vec4<f32>,
    interaction: vec4<f32>,
    interaction_motion: vec4<f32>,
    params: vec4<f32>,
    shading: vec4<f32>,
    visibility_range: vec4<f32>,
}

@group(2) @binding(0) var<uniform> tier: GrassTierUniform;

fn tier_params() -> GrassParams {
    var grass: GrassParams;
    grass.wind = tier.wind;
    grass.interaction = tier.interaction;
    grass.interaction_motion = tier.interaction_motion;
    grass.params = tier.params;
    grass.shading = tier.shading;
    return grass;
}

@vertex
fn vertex(vertex: GrassVertex) -> GrassVertexOutput {
    let identity = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
    let instance_matrix = calc_instance_world_matrix(
        vertex.i_pos_scale,
        vertex.i_rotation,
        identity,
    );
    return grass_vertex(
        vertex,
        instance_matrix,
        vec4<f32>(1.0, 1.0, 1.0, 1.0),
        tier.visibility_range,
        tier_params(),
    );
}

@fragment
fn fragment(
    in: GrassVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    return grass_fragment(in, is_front, tier_params());
}
