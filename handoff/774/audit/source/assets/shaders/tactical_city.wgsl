// Canonical client-generated vertices are shared across building instances.
#import bevy_pbr::mesh_view_bindings::view
#ifndef PREPASS_PIPELINE
#import bevy_pbr::{pbr_types, pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing, calculate_view}, mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT}
#endif
struct Building { transform: mat4x4<f32>, bounds: vec4<f32>, levels: vec4<u32>, uv_offset: vec4<f32> }
struct Surface { color: vec4<f32>, properties: vec4<f32>, uv_scale_offset: vec4<f32> }
@group(3) @binding(0) var<uniform> surface: Surface;
@group(3) @binding(1) var albedo: texture_2d<f32>;
@group(3) @binding(2) var albedo_sampler: sampler;
@group(3) @binding(3) var normal_map: texture_2d<f32>;
@group(3) @binding(4) var normal_sampler: sampler;
@group(2) @binding(0) var<storage, read> vertices: array<vec4<f32>>;
@group(2) @binding(1) var<storage, read> buildings: array<Building>;
@group(2) @binding(2) var<storage, read> visible: array<vec2<u32>>;
struct DrawRange { geometry: vec4<u32>, instances: vec4<u32> }
@group(2) @binding(3) var<storage, read> ranges: array<DrawRange>;

@group(2) @binding(4) var<storage, read> indices: array<u32>;
@group(2) @binding(5) var<storage, read> owners: array<u32>;
struct CityFrame { world_from_city: mat4x4<f32>, normal_from_city: mat4x4<f32>, handedness: vec4<f32> }
@group(2) @binding(6) var<storage, read> city_frame: CityFrame;

struct CityVertex {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) @interpolate(flat) visibility: f32,
    @location(5) @interpolate(flat) handedness: f32,
}

@vertex
fn vertex(@builtin(vertex_index) index: u32, @builtin(instance_index) instance: u32) -> CityVertex {
    let cluster = visible[instance];
    let range = ranges[cluster.x];
    let job = range.geometry;
    let owner = owners[cluster.y / range.instances.y];
    let local_index = (cluster.y % range.instances.y) * range.instances.z + index;
    var out: CityVertex;
    if local_index >= job.z {
        out.position = vec4(2.0, 2.0, 2.0, 1.0);
        return out;
    }
    let start = indices[job.y + local_index] * 3u;
    let p = vertices[start];
    let n = vertices[start+1u];
    let t = vertices[start+2u];
    var transform = buildings[owner].transform;
    var uv_offset = vec2(0.0);
    if (job.w & 512u) != 0u {
        let component = buildings[job.w >> 10u];
        transform = transform * component.transform;
        uv_offset = component.uv_offset.xy;
    }
    out.visibility = 1.0;
#ifndef PREPASS_PIPELINE
    let object = buildings[owner];
    if object.levels.y == 1u {
        let world_centre = (city_frame.world_from_city * vec4(object.bounds.xyz, 1.0)).xyz;
        out.visibility = 1.0 - smoothstep(bitcast<f32>(object.levels.z), bitcast<f32>(object.levels.w), distance(world_centre, view.world_position.xyz));
    }
#endif
    let normal_transform = city_frame.normal_from_city * transform;
    transform = city_frame.world_from_city * transform;
    out.handedness = city_frame.handedness.x;
    out.world_position = transform * vec4(p.xyz, 1.0);
    out.position = view.clip_from_world * out.world_position;
    out.normal = normalize((normal_transform * vec4(n.xyz, 0.0)).xyz);
    out.tangent = vec4(normalize((transform * vec4(t.xyz, 0.0)).xyz), t.w * out.handedness);
    out.uv = (vec2(p.w, n.w) + uv_offset) * surface.uv_scale_offset.xy + surface.uv_scale_offset.zw;
    return out;
}

#ifdef PREPASS_PIPELINE
@fragment
fn fragment(in: CityVertex) {
    if textureSample(albedo, albedo_sampler, in.uv).a * surface.color.a < surface.properties.z { discard; }
}
#else
@fragment
fn fragment(in: CityVertex, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let noise = fract(52.9829189 * fract(dot(in.position.xy, vec2(0.06711056, 0.00583715))));
    if in.visibility < noise { discard; }
    let color = textureSample(albedo, albedo_sampler, in.uv) * surface.color;
    if color.a < surface.properties.z { discard; }
    var pbr = pbr_types::pbr_input_new();
    pbr.material.flags |= pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;
    pbr.flags = MESH_FLAGS_SHADOW_RECEIVER_BIT;
    pbr.world_position = in.world_position;
    pbr.frag_coord = in.position;
    pbr.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr.V = calculate_view(in.world_position, pbr.is_orthographic);
    let city_front = select(!front, front, in.handedness > 0.0);
    pbr.world_normal = normalize(select(-in.normal, in.normal, city_front));
    pbr.N = pbr.world_normal;
    if surface.properties.w > 0.5 {
        let tangent = normalize(in.tangent.xyz);
        let bitangent = cross(pbr.world_normal, tangent) * in.tangent.w;
        let mapped = textureSample(normal_map, normal_sampler, in.uv).xyz * 2.0 - 1.0;
        pbr.N = normalize(mat3x3(tangent, bitangent, pbr.world_normal) * mapped);
    }
    pbr.material.base_color = vec4(color.rgb, 1.0);
    pbr.material.perceptual_roughness = surface.properties.x;
    pbr.material.metallic = surface.properties.y;
    return main_pass_post_lighting_processing(pbr, apply_pbr_lighting(pbr));
}
#endif
