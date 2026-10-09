// Disparity to a point.
//
// A pinhole grid is the textbook case, Z = f B / d. The other grids measure
// disparity as an angle, and each has the triangle that goes with it:
//
// * epipolar, two centres a baseline B apart along x: with β the angle toward
//   the baseline, the distance from the baseline is ρ = B / (tan β_L - tan β_R)
//   and the range from the left eye is ρ / cos β_L;
// * omnidirectional, every direction seen from a circle of diameter B: the
//   two tangent rays meet at 2γ, so the level reach along the left ray is
//   (B/2) / tan γ, and the point is measured from the rig's centre.
//
// A disparity below `min_disparity` is further than the grid can tell apart
// from infinity, and is put at `max_range` along its ray.

struct Params {
    width: u32,
    height: u32,
    grid_kind: u32,
    geometry: u32,
    grid: vec4<f32>,
    baseline: f32,
    disparity_step: f32,
    max_range: f32,
    min_disparity: f32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> disparity: array<f32>;
@group(0) @binding(2) var<storage, read> confidence: array<f32>;
@group(0) @binding(3) var<storage, read_write> points: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> distance: array<f32>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.width || id.y >= params.height) { return; }
    let p = id.y * params.width + id.x;
    let d = disparity[p];
    if (d < 0.0) {
        points[p] = vec4<f32>(0.0);
        distance[p] = 0.0;
        return;
    }
    let size = vec2<f32>(f32(params.width), f32(params.height));
    let at = vec2<f32>(id.xy) + vec2<f32>(0.5);
    let g = params.grid;
    let ray = grid_ray(params.grid_kind, g, size, at);
    var point = ray * params.max_range;
    if (d >= params.min_disparity) {
        if (params.grid_kind == 0u) {
            let z = g.x * params.baseline / d;
            point = ray * (z / ray.z);
        } else if (params.geometry == 0u) {
            let beta_left = mix(g.x, g.y, at.x / size.x);
            let beta_right = beta_left - d * params.disparity_step;
            let apart = tan(beta_left) - tan(beta_right);
            if (beta_right > -1.5707 && apart > 1e-6) {
                let rho = params.baseline / apart;
                point = ray * (rho / cos(beta_left));
            }
        } else {
            let lon = mix(g.x, g.y, at.x / size.x);
            let lat = mix(g.z, g.w, at.y / size.y);
            let turn = 0.5 * d * params.disparity_step;
            let reach = 0.5 * params.baseline / tan(turn);
            let eye = -0.5 * params.baseline * vec3<f32>(cos(lon), 0.0, -sin(lon));
            point = eye + vec3<f32>(reach * sin(lon), -reach * tan(lat), reach * cos(lon));
        }
    }
    var range = length(point);
    if (range > params.max_range) {
        point = point * (params.max_range / range);
        range = params.max_range;
    }
    points[p] = vec4<f32>(point, confidence[p]);
    distance[p] = range;
}
