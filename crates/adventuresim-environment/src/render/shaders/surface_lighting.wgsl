#define_import_path adventuresim_environment::surface_lighting
#import bevy_pbr::{mesh_view_bindings::view, pbr_types, pbr_functions,
    mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT}
#import adventuresim_environment::forest::SurfaceResponse

// Shared original forward lighting, independent of the vertex representation.
fn light_surface(response: SurfaceResponse, alpha: f32, world: vec4<f32>, normal: vec3<f32>, fragment: vec4<f32>) -> vec4<f32> {
    var pbr_input: pbr_types::PbrInput = pbr_types::pbr_input_new();
    pbr_input.material.flags |= pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;
    pbr_input.material.base_color = vec4<f32>(response.albedo, alpha);
    pbr_input.material.perceptual_roughness = response.roughness;
    pbr_input.material.metallic = 0.0;
    pbr_input.diffuse_occlusion = vec3<f32>(response.occlusion);
    pbr_input.frag_coord = fragment;
    pbr_input.world_position = world;
    pbr_input.flags = MESH_FLAGS_SHADOW_RECEIVER_BIT;
    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr_input.world_normal = pbr_functions::prepare_world_normal(normal, false, false);
    pbr_input.N = normalize(pbr_input.world_normal);
    pbr_input.V = pbr_functions::calculate_view(world, pbr_input.is_orthographic);
    let lit = pbr_functions::apply_pbr_lighting(pbr_input);
    return pbr_functions::main_pass_post_lighting_processing(pbr_input, lit);
}
