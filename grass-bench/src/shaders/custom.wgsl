// Bench custom material, main pass. Vertex: bevy 0.19's mesh.wgsl vertex
// (the naga#2416 `vertex_no_morph.instance_index` reads kept) plus the grass
// bend for objects flagged as grass. Fragment: one sun (N·L, first cascade
// shadow) plus a diffuse sky cubemap, per-object parameters from the
// MeshTag-indexed table, and `discard` only in alpha-tested pipelines
// (MAY_DISCARD) or when FORCE_DISCARD asks for the early-Z-killing variant.

#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    mesh_view_bindings::{globals, view, lights},
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    shadows::fetch_directional_shadow,
    skinning,
    morph::{morph_position, morph_normal, morph_tangent},
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    view_transformations::position_world_to_clip,
}
#import bench::custom_bindings::{
    globals_u, base_texture, base_sampler, sky_cube, sky_sampler, tables, FLAG_LIT, grass_bend,
    grass_bend_map, card_vertex, KIND_GRASS, KIND_CARD, KIND_GRASS_MAP, KIND_CARD_CURVED,
    displacement_map, displacement_sampler, scorch_color, card_curve_uv,
}
#ifdef VISIBILITY_RANGE_DITHER
#import bench::custom_bindings::visibility_range_dither
#endif

#ifdef MORPH_TARGETS
fn morph_vertex(vertex_in: Vertex, instance_index: u32) -> Vertex {
    var vertex = vertex_in;
    let first_vertex = mesh[instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;
    let weight_count = bevy_pbr::morph::layer_count(instance_index);
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i, instance_index);
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

    let kind = tables.objects[min(mesh_functions::get_tag(vertex_no_morph.instance_index), 127u)].params.w;
    let is_grass = abs(kind - KIND_GRASS) < 0.5;
    let is_map = abs(kind - KIND_GRASS_MAP) < 0.5;
    // Curved cards are cards: same placement, same corners, same table. The
    // only thing that differs is what their fragment does with the uv.
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

#ifdef VERTEX_POSITIONS
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
        let card = card_vertex(out.world_position.xyz, out.world_normal, uv, globals.time);
        out.world_position = vec4<f32>(card.world, 1.0);
        out.uv = card.uv;
    }
#endif
#endif
    out.position = position_world_to_clip(out.world_position.xyz);
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
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

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
#ifdef VISIBILITY_RANGE_DITHER
    visibility_range_dither(in.position, in.visibility_range_dither);
#endif

    let obj = tables.objects[min(mesh_functions::get_tag(in.instance_index), 127u)];
    var color = obj.base_color;
#ifdef VERTEX_UVS_A
    // Curved cards bend the sprite by moving the point it is read from; the
    // sample itself stays a single, unbranched textureSample so the mip
    // derivatives keep coming from the neighbouring pixels' own uvs.
    var uv = in.uv;
#ifdef VERTEX_UVS_B
    if abs(obj.params.w - KIND_CARD_CURVED) < 0.5 {
        uv = card_curve_uv(in.uv, in.uv_b, in.world_position.xyz, globals.time);
    }
#endif
    color *= textureSample(base_texture, base_sampler, uv);
#endif
#ifdef VERTEX_COLORS
    color *= in.color;
#endif
    // Grass blades carry their per-blade hash in color.a; they are opaque.
    // Cards keep the atlas alpha.
    if abs(obj.params.w - KIND_GRASS) < 0.5 || abs(obj.params.w - KIND_GRASS_MAP) < 0.5 {
        color.a = 1.0;
    }
    // Scorched trails (map mode): the displacement map's w is the trail the
    // trample map kept, so grass a trail ran over is dead, burnt by the
    // scorch knob in globals_u.map.w. Explicit level: obj comes from a
    // storage buffer, so this branch is not uniform control flow.
    if abs(obj.params.w - KIND_GRASS_MAP) < 0.5 && globals_u.map.w > 0.0 {
        let burn_uv = (in.world_position.xz - globals_u.map.xy) * globals_u.map.z;
        let trail = textureSampleLevel(displacement_map, displacement_sampler, burn_uv, 0.0).w;
        color = vec4<f32>(scorch_color(color.rgb, trail * globals_u.map.w), color.a);
    }

#ifdef MAY_DISCARD
    if color.a < globals_u.alpha_cutoff {
        discard;
    }
#endif
#ifdef FORCE_DISCARD
    if color.a < globals_u.alpha_cutoff {
        discard;
    }
#endif

#ifdef VERTEX_NORMALS
    // Explicit-level cube samples: WebGPU forbids textureSample (implicit
    // derivatives) under the per-object metallic branch below, and the sky
    // cube has one mip anyway.
    if (globals_u.flags & FLAG_LIT) != 0u {
        var normal = normalize(in.world_normal);
        if !is_front {
            normal = -normal;
        }
        var shadow = 1.0;
        if lights.n_directional_lights > 0u
            && (lights.directional_lights[0].flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            let view_z = dot(vec4<f32>(
                view.view_from_world[0].z,
                view.view_from_world[1].z,
                view.view_from_world[2].z,
                view.view_from_world[3].z,
            ), in.world_position);
            shadow = fetch_directional_shadow(0u, in.world_position, normal, view_z, in.position.xy);
        }
        var n_dot_l = max(dot(normal, globals_u.sun_dir.xyz), 0.0);
        if obj.params.w > 0.5 {
            // Thin blades: wrapped diffuse, like the game's grass.
            n_dot_l = mix(0.36, 1.0, n_dot_l);
        }
        let sun = globals_u.sun_color.rgb * n_dot_l * shadow;
        let sky = textureSampleLevel(sky_cube, sky_sampler, normal, 0.0).rgb * globals_u.sky_strength;
        var light = sun + sky;
        let metallic = obj.params.x;
        if metallic > 0.0 {
            let view_vec = normalize(view.world_position - in.world_position.xyz);
            let refl_dir = normalize(mix(reflect(-view_vec, normal), normal, obj.params.y));
            let refl = textureSampleLevel(sky_cube, sky_sampler, refl_dir, 0.0).rgb * globals_u.sky_strength;
            light = mix(light, sun + refl, metallic);
        }
        color = vec4(color.rgb * light + sky * obj.params.z * metallic + obj.emissive.rgb, color.a);
    }
#endif

    var out: FragmentOutput;
    out.color = color;
    return out;
}
