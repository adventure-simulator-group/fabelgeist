// Camera depth-prepass fragment for tactical terrain.
//
// Bevy's standard prepass vertex stage positions the terrain; this fragment
// repeats the one discard in `tactical_terrain.wgsl` that changes coverage.
// Without it the coarse surface inside the camera-local detail patch writes
// depth that hides the refined surface beneath it.

#import bevy_pbr::{
    mesh_view_bindings::view,
    prepass_io::VertexOutput,
}

// Must match `TacticalTerrainMaterial` in `tactical_terrain.wgsl`.
struct TacticalTerrainMaterial {
    base_color: vec4<f32>,
    grass_color: vec4<f32>,
    cover: vec4<f32>,
    weather: vec4<f32>,
    far_sward: vec4<f32>,
    lod_sward: vec4<f32>,
    playable_bounds: vec4<f32>,
    detail_patch: vec4<f32>,
    soil_detail: vec4<f32>,
    litter_detail: vec4<f32>,
    cliff_palette_a: vec4<f32>,
    cliff_palette_b: vec4<f32>,
    cliff_surface: vec4<f32>,
    cliff_structure_a: vec4<f32>,
    cliff_structure_b: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> terrain: TacticalTerrainMaterial;

#ifdef PREPASS_FRAGMENT
#import bevy_pbr::prepass_io::FragmentOutput

// Shadow views reach here only to emulate unclipped orthographic depth on
// adapters without depth-clip control. Keep Bevy's standard behaviour.
@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.frag_depth = in.unclipped_depth;
#endif
    return out;
}
#else
// The camera's depth prepass, attached by the material's `specialize`.
@fragment
fn fragment(in: VertexOutput) {
    if terrain.detail_patch.x > 0.5
        && distance(in.world_position.xz, view.lod_view_world_position.xz) < terrain.detail_patch.y {
        discard;
    }
}
#endif
