// The original analytic leaf outline and fragment-only sample coverage.
#define_import_path adventuresim_environment::leaf_surface
#import adventuresim_environment::forest::{Species, SurfaceResponse, linear}

// The blade's outline, positive inside: an ovate toothed blade, a needle
// spray frayed at its edge, or a frond's leaflets.
fn leaf_shape(uv: vec2<f32>, foliage: u32, seed: f32) -> f32 {
    let x = abs(uv.x - 0.5);
    let y = uv.y;
    let b = clamp((y - 0.1) / 0.9, 0.0, 1.0);
    let half = 0.5 * pow(sin(3.14159 * pow(b, 0.7)), 0.85) * (1.0 - 0.15 * b);
    let teeth = 0.012 * abs(fract(b * 14.0 + seed) - 0.5);
    let blade = select(0.018 - x, half - teeth - x, y > 0.1);
    let broad = min(blade, min(y, 1.0 - y) * 0.5);
    let reach = 0.5 * pow(sin(3.14159 * min(y * 1.05, 1.0)), 0.4);
    let needle = abs(fract(y * 15.0 - x * 5.0 + seed * 3.0) - 0.5);
    let needles = min(reach - x, max(reach * 0.4 - x, (0.4 - needle) * 0.05));
    let leaflet = abs(fract(y * 22.0 - x * 4.0 + seed) - 0.5);
    let fronds = max(0.03 - x, min(0.48 - x, (0.3 - leaflet) * 0.06));
    if (foliage == 1u) {
        return needles;
    } else if (foliage == 2u) {
        return fronds;
    }
    return broad;
}

// How much of the pixel the outline covers: its distance over its rate of
// change across the pixel. Under multisampling it becomes sample coverage,
// so sub-pixel needles add up rather than shimmer.
fn coverage(sdf: f32) -> f32 {
    return clamp(sdf / max(fwidth(sdf), 1e-5) + 0.5, 0.0, 1.0);
}

// Original source leaf pigment and simulated/crown-depth occlusion.
fn leaf_response(kind: Species, uv: vec2<f32>, seed: f32, surface: vec3<f32>, crown: vec4<f32>, light: f32) -> SurfaceResponse {
    // Every blade its own shade of the species' green: some yellower, some
    // darker; the midrib paler.
    let r = seed;
    var albedo = linear(kind.leaf.rgb);
    albedo = mix(vec3<f32>(dot(albedo, vec3<f32>(0.3, 0.6, 0.1))), albedo, 0.8);
    albedo = mix(albedo, albedo * vec3<f32>(1.3, 1.1, 0.6), fract(r * 5.3) * 0.5);
    albedo *= 0.75 + 0.45 * fract(r * 11.7);
    let midrib = 1.0 - smoothstep(0.0, 0.025, abs(uv.x - 0.5));
    albedo = mix(albedo, albedo * vec3<f32>(1.6, 1.5, 1.1), midrib * 0.4);
    // Deep in a crown little sky gets through: the light the simulation
    // measured, and how far in the leaf sits.
    let depth = clamp(length(surface - crown.xyz) / max(crown.w, 1e-3), 0.0, 1.0);
    let ao = mix(0.3, 1.0, light) * mix(0.45, 1.0, smoothstep(0.1, 1.0, depth));

    var response: SurfaceResponse;
    response.albedo = albedo;
    response.occlusion = ao;
    response.roughness = 0.6;
    return response;
}
