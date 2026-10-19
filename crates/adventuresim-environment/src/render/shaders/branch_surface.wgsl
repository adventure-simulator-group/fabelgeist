// The original grown bark coordinates and pattern, evaluated per fragment.
#define_import_path adventuresim_environment::branch_surface
#import adventuresim_environment::forest::{Species, SurfaceResponse, linear, TAU, Branch, basis, bark_noise}
#import adventuresim_environment::branch_geometry::axis_at

// Where a point lies on its segment: `u` a turn around the axis, `v` along
// it in turns.
fn bark_coordinates(branch: Branch, x: vec3<f32>) -> vec2<f32> {
    let p0 = branch.a.xyz;
    let chord = branch.b.xyz - p0;
    let len = max(length(chord), 1e-6);
    var s = clamp(dot(x - p0, chord) / (len * len), 0.0, 1.0);
    var point = axis_at(branch, s);
    for (var i = 0; i < 2; i++) {
        s = clamp(s + dot(x - point.center, point.tangent) / len, 0.0, 1.0);
        point = axis_at(branch, s);
    }
    let d = x - point.center;
    let along = dot(d, point.tangent);
    let radial = d - point.tangent * along;
    let out_dir = radial / max(length(radial), 1e-9);
    let frame = basis(point.tangent);
    let u = fract(atan2(dot(out_dir, frame[1]), dot(out_dir, frame[0])) / TAU + 1.0);
    let v = mix(branch.v_start, branch.v_end, s + along / len);
    return vec2<f32>(u, v);
}

// Bark at `(u, v)`: the albedo's factor, and how open the surface is. Ridges
// run along the branch with dark furrows between; a palm's stem is ringed by
// the scars of fallen fronds.
fn bark(u: f32, v: f32, palm: bool) -> vec2<f32> {
    let wander = bark_noise(vec2<f32>(u * 6.0, v * 2.0), 6.0);
    var ridge = abs(fract(u * 8.0 + (wander - 0.5) * 1.2) - 0.5) * 2.0;
    var furrow_depth = 0.45;
    if (palm) {
        ridge = abs(fract(v * 3.0 + (wander - 0.5) * 0.3) - 0.5) * 2.0;
        ridge = mix(1.0, ridge, 0.6 + 0.4 * bark_noise(vec2<f32>(u * 5.0, v * 40.0), 5.0));
        furrow_depth = 0.75;
    }
    let grain = bark_noise(vec2<f32>(u * 32.0, v * 12.0), 32.0);
    let patches = bark_noise(vec2<f32>(u * 3.0, v * 1.2), 3.0);
    let furrow = smoothstep(0.0, 0.35, ridge) * (0.7 + 0.3 * grain);
    return vec2<f32>(mix(furrow_depth, 1.1, furrow) * (0.8 + 0.4 * patches), furrow);
}

// Original source wood response, shared by live and expanded geometry.
fn branch_response(branch: Branch, kind: Species, surface: vec3<f32>, light: f32) -> SurfaceResponse {
    let palm = u32(kind.leaf.w) == 2u;
    let place = bark_coordinates(branch, surface);
    let pattern = bark(place.x, place.y, palm);
    let albedo = linear(kind.bark.rgb) * pattern.x;
    // Deep in a crown, and in the furrows, less sky reaches the bark.
    let ao = mix(0.5, 1.0, light) * mix(0.7, 1.0, pattern.y);

    var response: SurfaceResponse;
    response.albedo = albedo;
    response.occlusion = ao;
    response.roughness = 0.9;
    return response;
}
