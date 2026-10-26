// Chunk-instanced distant loose-stone billboards.
//
// A port of `tactical_pebble_billboard.wgsl` onto the chunk instance buffer.
// The patch mesh still stores each pebble's centre in the unused normal channel
// and its positions as quad offsets, so every member of a patch faces the camera
// independently; the only change is that the patch's world matrix now comes
// from the instance buffer rather than a per-entity mesh uniform.

// `fwidth`/`dpdx` appear inside branches below, which browser WebGPU rejects
// under uniformity analysis unless the diagnostic is disabled here.
diagnostic(off, derivative_uniformity);

#ifdef PREPASS_PIPELINE
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::view_transformations::position_world_to_clip
#else
#import bevy_pbr::{
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
}
#endif

#import fablegeist::chunk_instancing::{
    Vertex, chunk_band, instance_world_matrix, visibility_range_dither_level,
}

struct TacticalPebbleBillboardInstancedMaterial {
    color: vec4<f32>,
    lighting: vec4<f32>,
    ambient: vec4<f32>,
}

@group(3) @binding(0) var<uniform> pebble: TacticalPebbleBillboardInstancedMaterial;

struct BillboardVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
#ifdef VISIBILITY_RANGE_DITHER
    @location(0) @interpolate(flat) visibility_range_dither: i32,
#endif
    @location(1) world_position: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> BillboardVertexOutput {
    var out: BillboardVertexOutput;
    let instance_matrix = instance_world_matrix(vertex.i_pos_scale, vertex.i_rotation);

    var centre_local = vec3<f32>(0.0, 0.0, 0.0);
#ifdef VERTEX_NORMALS
    centre_local = vertex.normal;
#endif
    let centre_world = (instance_matrix * vec4<f32>(centre_local, 1.0)).xyz;
    let to_camera = normalize(view.world_position.xz - centre_world.xz + vec2<f32>(0.0001, 0.0));
    let right = vec3<f32>(to_camera.y, 0.0, -to_camera.x);
    // Quad offsets are authored in patch-local metres, so the instance's
    // uniform scale has to reach them even though they bypass the matrix.
    let quad_scale = vertex.i_pos_scale.w;
    let world_position = vec4<f32>(
        centre_world
            + right * (vertex.position.x * quad_scale)
            + vec3<f32>(0.0, vertex.position.y * quad_scale, 0.0),
        1.0,
    );

    out.world_position = world_position;
    out.clip_position = position_world_to_clip(world_position.xyz);
#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#else
    out.uv = vec2<f32>(0.5, 0.5);
#endif

#ifdef VISIBILITY_RANGE_DITHER
    let instance_origin = instance_matrix * vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.visibility_range_dither =
        visibility_range_dither_level(chunk_band.visibility_range, instance_origin.xyz);
#endif
    return out;
}

fn interleaved_gradient_noise(pixel: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

#ifdef PREPASS_PIPELINE
// The camera's depth prepass repeats every main-pass discard; the main pass
// writes alpha 1.0 wherever it keeps a pixel.
@fragment
fn fragment(in: BillboardVertexOutput) {
#ifdef VISIBILITY_RANGE_DITHER
    if in.visibility_range_dither != 0 {
        let magnitude = clamp(f32(abs(in.visibility_range_dither)) / 16.0, 0.0, 1.0);
        let pixel_hash = interleaved_gradient_noise(floor(in.clip_position.xy));
        if in.visibility_range_dither > 0 {
            if pixel_hash < magnitude {
                discard;
            }
        } else if pixel_hash >= 1.0 - magnitude {
            discard;
        }
    }
#endif
    let shape = in.uv * 2.0 - vec2<f32>(1.0);
    let radius_squared = dot(shape, shape);
    let edge_width = max(fwidth(radius_squared), 0.001);
    let silhouette = 1.0 - smoothstep(1.0 - edge_width, 1.0 + edge_width, radius_squared);

    // UV derivatives give the quad's rasterized diameter, so each physical size
    // fades independently instead of sharing a world-distance cutoff.
    let uv_per_pixel = max(length(dpdx(in.uv)), length(dpdy(in.uv)));
    let diameter_pixels = 1.0 / max(uv_per_pixel, 0.00001);
    let screen_coverage = smoothstep(0.75, 1.5, diameter_pixels);
    let coverage = silhouette * screen_coverage;
    if interleaved_gradient_noise(floor(in.clip_position.xy)) > coverage {
        discard;
    }

}
#else
@fragment
fn fragment(in: BillboardVertexOutput) -> @location(0) vec4<f32> {
#ifdef VISIBILITY_RANGE_DITHER
    if in.visibility_range_dither != 0 {
        let magnitude = clamp(f32(abs(in.visibility_range_dither)) / 16.0, 0.0, 1.0);
        let pixel_hash = interleaved_gradient_noise(floor(in.clip_position.xy));
        if in.visibility_range_dither > 0 {
            if pixel_hash < magnitude {
                discard;
            }
        } else if pixel_hash >= 1.0 - magnitude {
            discard;
        }
    }
#endif
    let shape = in.uv * 2.0 - vec2<f32>(1.0);
    let radius_squared = dot(shape, shape);
    let edge_width = max(fwidth(radius_squared), 0.001);
    let silhouette = 1.0 - smoothstep(1.0 - edge_width, 1.0 + edge_width, radius_squared);

    // UV derivatives give the quad's rasterized diameter, so each physical size
    // fades independently instead of sharing a world-distance cutoff.
    let uv_per_pixel = max(length(dpdx(in.uv)), length(dpdy(in.uv)));
    let diameter_pixels = 1.0 / max(uv_per_pixel, 0.00001);
    let screen_coverage = smoothstep(0.75, 1.5, diameter_pixels);
    let coverage = silhouette * screen_coverage;
    if interleaved_gradient_noise(floor(in.clip_position.xy)) > coverage {
        discard;
    }

    let to_camera = normalize(view.world_position.xz - in.world_position.xz + vec2<f32>(0.0001, 0.0));
    let right = vec3<f32>(to_camera.y, 0.0, -to_camera.x);
    let facing = vec3<f32>(to_camera.x, 0.0, to_camera.y);
    let surface_normal = normalize(
        right * shape.x
        + vec3<f32>(0.0, shape.y, 0.0)
        + facing * sqrt(max(1.0 - radius_squared, 0.0)),
    );
    let direct = max(dot(surface_normal, normalize(pebble.lighting.xyz)), 0.0)
        * pebble.lighting.w
        * (1.0 - pebble.ambient.w);
    let irradiance = pebble.ambient.rgb * pebble.ambient.w + vec3<f32>(direct);
    return vec4<f32>(pebble.color.rgb * irradiance, 1.0);
}
#endif
