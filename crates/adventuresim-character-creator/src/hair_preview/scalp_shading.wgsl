// The scalp under the hair, shaded as the hair is: the standard material's
// lighting, with each point's normal that of the topmost strand painted
// there, read from the baked strands texture -- its direction across the
// texture, turned onto the scalp by the cap's tangents -- and the
// environment's reflection faded by the strands' shade, as hair deep in the
// groom is. Where no strand runs, the scalp is lit by its own normal.

#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#import adventuresim::strand::strand_normal

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var strands_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var strands_sampler: sampler;

// How long the filtered direction must be to be a strand's: filtering
// averages strands running different ways, and where none runs it is none.
const SETTLED: f32 = 0.5;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let face = pbr_input.N;
    var normal = face;
    var shade = 1.0;
#ifdef VERTEX_UVS_A
    let painted = textureSample(strands_texture, strands_sampler, in.uv);
    shade = painted.b;
#ifdef VERTEX_TANGENTS
    let along = painted.rg * 2.0 - 1.0;
    let settled = smoothstep(0.0, SETTLED, length(along));
    if (settled > 0.0) {
        let across_u = normalize(in.world_tangent.xyz);
        let across_v = cross(normalize(in.world_normal), across_u) * in.world_tangent.w;
        let tangent = normalize(across_u * along.x + across_v * along.y);
        let eye = normalize(view.world_position - in.world_position.xyz);
        normal = normalize(mix(face, strand_normal(tangent, eye, face), settled));
    }
#endif
#endif
    pbr_input.N = normal;
    pbr_input.world_normal = normal;
    pbr_input.specular_occlusion *= shade;
    return main_pass_post_lighting_processing(pbr_input, apply_pbr_lighting(pbr_input));
}
