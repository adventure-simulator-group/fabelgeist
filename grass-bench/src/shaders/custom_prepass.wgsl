// Prepass for the bench custom material. Vertex: bevy's prepass.wgsl vertex
// plus the grass bend for flagged objects, so depth / shadows / motion
// vectors match the main pass. Fragment: bevy's prepass fragment plus the
// same alpha test as custom.wgsl, so cut-out leaves stay cut out.

#import bevy_pbr::{
    prepass_bindings,
    mesh_bindings::mesh,
    mesh_functions,
    prepass_io::{Vertex, VertexOutput},
    skinning,
    morph,
    morph::{morph_position, morph_normal, morph_tangent},
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
}
#import bevy_render::globals::Globals
#import bench::custom_bindings::{
    globals_u, base_texture, base_sampler, objects, grass_bend, grass_bend_map, card_vertex,
    card_curve_uv, KIND_GRASS, KIND_CARD, KIND_GRASS_MAP, KIND_CARD_CURVED,
}
#ifdef VISIBILITY_RANGE_DITHER
#import bench::custom_bindings::visibility_range_dither
#endif

// The prepass / shadow view layout carries the globals at binding 1 (the
// main pass has them at 11, which mesh_view_bindings::globals would declare).
@group(0) @binding(1) var<uniform> globals: Globals;
// prepass_io only defines FragmentOutput when the pipeline has outputs
// (normals / motion vectors / deferred); depth-only prepasses and shadow
// maps compile the fragment below without any return value.
#ifdef PREPASS_FRAGMENT
#import bevy_pbr::prepass_io::FragmentOutput
#endif

#ifdef MORPH_TARGETS
fn morph_vertex(vertex_in: Vertex, instance_index: u32) -> Vertex {
    var vertex = vertex_in;
    let first_vertex = mesh[instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;
    let weight_count = morph::layer_count(instance_index);
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = morph::weight_at(i, instance_index);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph_position(vertex_index, i, instance_index);
#ifdef VERTEX_NORMALS
        vertex.normal += weight * morph_normal(vertex_index, i, instance_index);
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent += vec4(weight * morph_tangent(vertex_index, i, instance_index), 0.0);
#endif
    }
    return vertex;
}

fn morph_prev_vertex(vertex_in: Vertex, instance_index: u32) -> Vertex {
    var vertex = vertex_in;
    let first_vertex = mesh[instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;
    let weight_count = morph::layer_count(instance_index);
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = morph::prev_weight_at(i, instance_index);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph_position(vertex_index, i, instance_index);
    }
    return vertex;
}
#endif

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph, vertex_no_morph.instance_index);
#else
    var vertex = vertex_no_morph;
#endif

    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);

#ifdef SKINNED
    var world_from_local = skinning::skin_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex_no_morph.instance_index
    );
#else
    var world_from_local = mesh_world_from_local;
#endif

    let kind = objects[mesh_functions::get_tag(vertex_no_morph.instance_index)].params.w;
    let is_grass = abs(kind - KIND_GRASS) < 0.5;
    let is_map = abs(kind - KIND_GRASS_MAP) < 0.5;
    let is_card = abs(kind - KIND_CARD) < 0.5 || abs(kind - KIND_CARD_CURVED) < 0.5;
    var blade_hash = 0.0;
#ifdef VERTEX_COLORS
    blade_hash = vertex.color.a;
#endif
    var uv = vec2<f32>(0.5, 0.0);
#ifdef VERTEX_UVS_A
    uv = vertex.uv;
#endif
    if is_grass || is_map {
#ifdef VERTEX_UVS_B
        // Thickness: the root pair sits at uv.y = 0 and uv_b holds the blade
        // root, so scaling their offset from it widens the blade; the tip
        // (uv.y = 1) keeps its lean.
        if uv.y < 0.5 {
            vertex.position = vec3<f32>(
                vertex.uv_b.x + (vertex.position.x - vertex.uv_b.x) * globals_u.blade_width,
                vertex.position.y,
                vertex.uv_b.y + (vertex.position.z - vertex.uv_b.y) * globals_u.blade_width,
            );
        }
#endif
    }

    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    if is_grass {
        out.world_position = vec4<f32>(
            grass_bend(out.world_position.xyz, uv, blade_hash, globals.time), 1.0);
    } else if is_map {
        out.world_position = vec4<f32>(grass_bend_map(out.world_position.xyz, uv, blade_hash, globals.time), 1.0);
    }
#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#ifdef VERTEX_NORMALS
    if is_card {
        let card = card_vertex(
            out.world_position.xyz,
            mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex_no_morph.instance_index),
            uv,
            globals.time,
        );
        out.world_position = vec4<f32>(card.world, 1.0);
        out.uv = card.uv;
    }
#endif
#endif
    out.position = position_world_to_clip(out.world_position.xyz);
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.unclipped_depth = out.position.z;
    out.position.z = min(out.position.z, 1.0);
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef NORMAL_PREPASS_OR_DEFERRED_PREPASS
#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(world_from_local, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex_no_morph.instance_index
    );
#endif
#endif
#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef MOTION_VECTOR_PREPASS
#ifdef MORPH_TARGETS
#ifdef HAS_PREVIOUS_MORPH
    let prev_vertex = morph_prev_vertex(vertex_no_morph, vertex_no_morph.instance_index);
#else
    let prev_vertex = vertex_no_morph;
#endif
#else
    let prev_vertex = vertex_no_morph;
#endif

#ifdef SKINNED
#ifdef HAS_PREVIOUS_SKIN
    let prev_model = skinning::skin_prev_model(
        prev_vertex.joint_indices,
        prev_vertex.joint_weights,
        vertex_no_morph.instance_index
    );
#else
    let prev_model = mesh_functions::get_previous_world_from_local(vertex_no_morph.instance_index);
#endif
#else
    let prev_model = mesh_functions::get_previous_world_from_local(vertex_no_morph.instance_index);
#endif

    out.previous_world_position = mesh_functions::mesh_position_local_to_world(
        prev_model,
        vec4<f32>(prev_vertex.position, 1.0)
    );
    if is_grass {
        // Same bend at the previous frame's time: the sway is motion, the
        // object is not.
        out.previous_world_position = vec4<f32>(
            grass_bend(out.previous_world_position.xyz, uv, blade_hash, globals.time - globals.delta_time), 1.0);
    }
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex_no_morph.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex_no_morph.instance_index, mesh_world_from_local[3]);
#endif

    return out;
}

fn alpha_test(in: VertexOutput) {
    let obj = objects[mesh_functions::get_tag(in.instance_index)];
    var alpha = obj.base_color.a;
    // Sampled before the per-object branch: WebGPU forbids textureSample
    // (implicit derivatives) in non-uniform control flow.
#ifdef VERTEX_UVS_A
    // The same bend the main pass applies, or the prepass would cut the
    // depth of an unbent sprite out of a bent one.
    var uv = in.uv;
#ifdef VERTEX_UVS_B
    if abs(obj.params.w - KIND_CARD_CURVED) < 0.5 {
        uv = card_curve_uv(in.uv, in.uv_b, in.world_position.xyz, globals.time);
    }
#endif
    alpha *= textureSample(base_texture, base_sampler, uv).a;
#endif
#ifdef VERTEX_COLORS
    alpha *= in.color.a;
#endif
    // Grass blades are opaque; their color.a is the blade hash.
    if abs(obj.params.w - KIND_GRASS) < 0.5 || abs(obj.params.w - KIND_GRASS_MAP) < 0.5 {
        return;
    }
    if alpha < globals_u.alpha_cutoff {
        discard;
    }
}

#ifndef PREPASS_FRAGMENT
@fragment
fn fragment(in: VertexOutput) {
#ifdef VISIBILITY_RANGE_DITHER
    visibility_range_dither(in.position, in.visibility_range_dither);
#endif
#ifdef MAY_DISCARD
    alpha_test(in);
#endif
#ifdef FORCE_DISCARD
    alpha_test(in);
#endif
}
#else
@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
#ifdef VISIBILITY_RANGE_DITHER
    visibility_range_dither(in.position, in.visibility_range_dither);
#endif
#ifdef MAY_DISCARD
    alpha_test(in);
#endif
#ifdef FORCE_DISCARD
    alpha_test(in);
#endif

    var out: FragmentOutput;

#ifdef NORMAL_PREPASS
    out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);
#endif

#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.frag_depth = in.unclipped_depth;
#endif

#ifdef MOTION_VECTOR_PREPASS
    let clip_position_t = view.unjittered_clip_from_world * in.world_position;
    let clip_position = clip_position_t.xy / clip_position_t.w;
    let previous_clip_position_t = prepass_bindings::previous_view_uniforms.clip_from_world * in.previous_world_position;
    let previous_clip_position = previous_clip_position_t.xy / previous_clip_position_t.w;
    out.motion_vector = (clip_position - previous_clip_position) * vec2(0.5, -0.5);
#endif

    return out;
}
#endif // PREPASS_FRAGMENT
