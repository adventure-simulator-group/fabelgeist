// What every stereo kernel that thinks about rays needs: the rectified grid,
// in both directions, and each source projection. `rig.rs` is the CPU twin of
// every function here.
//
// Coordinates are x right, y down, z forward. Pixel centres are at
// half-integers.

const PI: f32 = 3.14159265358979;
const TAU: f32 = 6.28318530717959;

fn angles_direction(longitude: f32, latitude: f32) -> vec3<f32> {
    return vec3<f32>(cos(latitude) * sin(longitude), -sin(latitude), cos(latitude) * cos(longitude));
}

// The ray through `p` in a grid of `size` pixels.
//   kind 0: pinhole, g = (f, cx, cy, _)
//   kind 1: epipolar, g = (left, right, top, bottom), angles about the baseline
//   kind 2: panorama, g = (left, right, top, bottom), longitude and latitude
fn grid_ray(kind: u32, g: vec4<f32>, size: vec2<f32>, p: vec2<f32>) -> vec3<f32> {
    if (kind == 0u) {
        return normalize(vec3<f32>((p.x - g.y) / g.x, (p.y - g.z) / g.x, 1.0));
    }
    let across = mix(g.x, g.y, p.x / size.x);
    let down = mix(g.z, g.w, p.y / size.y);
    if (kind == 1u) {
        let alpha = -down;
        return vec3<f32>(sin(across), cos(across) * sin(alpha), cos(across) * cos(alpha));
    }
    return angles_direction(across, down);
}

// Where a direction lands on the grid: xy in pixels, z 1 when it is on it.
fn grid_pixel(kind: u32, g: vec4<f32>, size: vec2<f32>, d: vec3<f32>) -> vec3<f32> {
    var p = vec2<f32>(-1.0);
    if (kind == 0u) {
        if (d.z <= 1e-6) { return vec3<f32>(0.0); }
        p = vec2<f32>(g.y + g.x * d.x / d.z, g.z + g.x * d.y / d.z);
    } else if (kind == 1u) {
        let beta = asin(clamp(d.x, -1.0, 1.0));
        let vertical = -atan2(d.y, d.z);
        p = vec2<f32>((beta - g.x) / (g.y - g.x) * size.x, (vertical - g.z) / (g.w - g.z) * size.y);
    } else {
        var lon = atan2(d.x, d.z);
        lon = g.x + fract((lon - g.x) / TAU) * TAU;
        let lat = asin(clamp(-d.y, -1.0, 1.0));
        p = vec2<f32>((lon - g.x) / (g.y - g.x) * size.x, (lat - g.z) / (g.w - g.z) * size.y);
    }
    let inside = p.x >= 0.0 && p.y >= 0.0 && p.x < size.x && p.y < size.y;
    return vec3<f32>(p, select(0.0, 1.0, inside));
}

// Where a point, measured from the rig's centre, appears to one eye of
// omnidirectional stereo.
fn ods_pixel(g: vec4<f32>, size: vec2<f32>, half_ipd: f32, point: vec3<f32>, left: bool) -> vec3<f32> {
    let level = length(point.xz);
    if (level <= half_ipd) { return vec3<f32>(0.0); }
    let turn = asin(half_ipd / level);
    let lon = atan2(point.x, point.z) + select(-turn, turn, left);
    let reach = sqrt(level * level - half_ipd * half_ipd);
    let lat = atan2(-point.y, reach);
    return grid_pixel(2u, g, size, angles_direction(lon, lat));
}

// A fisheye's radius, in focal lengths, of a ray `theta` off its axis.
fn fisheye_radius(model: u32, theta: f32) -> f32 {
    switch model {
        case 1u: { return 2.0 * sin(theta * 0.5); }
        case 2u: { return 2.0 * tan(theta * 0.5); }
        case 3u: { return sin(theta); }
        default: { return theta; }
    }
}

// The angle off a fisheye's axis of a ray at `radius` focal lengths.
fn fisheye_angle(model: u32, radius: f32) -> f32 {
    switch model {
        case 1u: { return 2.0 * asin(clamp(radius * 0.5, -1.0, 1.0)); }
        case 2u: { return 2.0 * atan(radius * 0.5); }
        case 3u: { return asin(clamp(radius, -1.0, 1.0)); }
        default: { return radius; }
    }
}

// The direction in an eye's camera that a place in its region was seen along:
// xyz, and w 1 when the region holds picture there.
//   kind 0: pinhole, a = (fx, fy, cx, cy), b = (k1, k2, p1, p2), c = (k3, ...)
//   kind 1: equirectangular, a = (lon min, lon max, lat min, lat max)
//   kind 2: fisheye, a = (cx, cy, rx, ry), b = (half fov, model, radius(half fov), _)
fn unproject_source(kind: u32, a: vec4<f32>, b: vec4<f32>, c: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    if (kind == 0u) {
        let xd = (uv.x - a.z) / a.x;
        let yd = (uv.y - a.w) / a.y;
        var x = xd;
        var y = yd;
        // Distortion has no closed-form inverse; a fixed number of steps of
        // the fixed point keeps every pixel's cost the same.
        for (var i = 0; i < 12; i++) {
            let r2 = x * x + y * y;
            let radial = 1.0 + b.x * r2 + b.y * r2 * r2 + c.x * r2 * r2 * r2;
            let dx = 2.0 * b.z * x * y + b.w * (r2 + 2.0 * x * x);
            let dy = b.z * (r2 + 2.0 * y * y) + 2.0 * b.w * x * y;
            x = (xd - dx) / radial;
            y = (yd - dy) / radial;
        }
        return vec4<f32>(normalize(vec3<f32>(x, y, 1.0)), 1.0);
    }
    if (kind == 1u) {
        let lon = mix(a.x, a.y, uv.x);
        let lat = a.w - uv.y * (a.w - a.z);
        return vec4<f32>(angles_direction(lon, lat), 1.0);
    }
    let dx = (uv.x - a.x) / a.z;
    let dy = (uv.y - a.y) / a.w;
    let g = sqrt(dx * dx + dy * dy);
    if (g > 1.0) { return vec4<f32>(0.0); }
    let theta = fisheye_angle(u32(b.y), g * b.z);
    let phi = atan2(dy, dx);
    return vec4<f32>(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta), 1.0);
}
