// Depth back on an eye's own picture.
//
// Matching happens on the rectified grid, but what a player draws, and what a
// headset shows each eye, is the picture the file holds: an equirectangular
// half-sphere, a fisheye circle, a distorted pinhole frame. This runs once per
// pixel of that picture:
//
// 1. the pixel's ray, through the inverse of the eye's projection;
// 2. that ray's place on the rectified grid;
// 3. the surface along it. For the left eye the grid is its own, so the
//    points are read there -- blended between neighbours only when they agree
//    on range, so a silhouette stays a silhouette. For the right eye the grid
//    holds the left eye's points, so the row is searched for the ones that
//    land on this pixel when seen from the right, the pair either side of it
//    interpolated, and the nearest surface kept -- which is what the right eye
//    would have seen;
// 4. the point measured from the eye's own centre, in the eye's own camera.
//
// A pixel whose ray leaves the grid, or whose surface the left eye never saw,
// is left empty rather than guessed.

struct Params {
    width: u32,
    height: u32,
    grid_width: u32,
    grid_height: u32,
    grid_kind: u32,
    geometry: u32,
    projection_kind: u32,
    eye: u32,
    grid: vec4<f32>,
    projection_a: vec4<f32>,
    projection_b: vec4<f32>,
    projection_c: vec4<f32>,
    eye_from_grid: mat4x4<f32>,
    baseline: f32,
    inverse_tolerance: f32,
    disparities: u32,
    wraps: u32,
    // Zero, or the distance in metres of a backdrop that every pixel of the
    // eye's picture without a trustworthy depth is put on -- one the matcher
    // rejected, one outside the grid, or one too far away for the baseline to
    // tell apart from infinity.
    backdrop_range: f32,
    // Points nearer than this in inverse metres count as too far to trust.
    backdrop_inverse: f32,
    // And points matched with less confidence than this count as untrusted
    // too: a sky with nothing to match in produces near disparities that pass
    // the checks, only just.
    backdrop_confidence: f32,
}

// The confidence a backdrop point is marked with: there, so it is drawn, but
// nothing anyone matched.
const BACKDROP: f32 = 0.001;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> view_points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> view_distance: array<f32>;

fn grid_size() -> vec2<f32> {
    return vec2<f32>(f32(params.grid_width), f32(params.grid_height));
}

// A grid pixel's point, wrapping across the seam of a whole panorama.
fn fetch(x: i32, y: i32) -> vec4<f32> {
    let w = i32(params.grid_width);
    var column = x;
    if (params.wraps == 1u) {
        column = ((x % w) + w) % w;
    }
    if (column < 0 || y < 0 || column >= w || y >= i32(params.grid_height)) { return vec4<f32>(0.0); }
    return points[u32(y) * params.grid_width + u32(column)];
}

// How far apart two columns are, the short way round a whole panorama.
fn apart(a: f32, b: f32) -> f32 {
    var d = a - b;
    if (params.wraps == 1u) {
        let w = f32(params.grid_width);
        d = d - w * round(d / w);
    }
    return d;
}

fn inverse_range(p: vec4<f32>) -> f32 {
    return 1.0 / max(length(p.xyz), 1e-4);
}

// The centre a ray in direction `d` (grid frame) leaves this eye from.
fn eye_centre(d: vec3<f32>) -> vec3<f32> {
    if (params.geometry == 0u) {
        return select(vec3<f32>(0.0), vec3<f32>(params.baseline, 0.0, 0.0), params.eye == 1u);
    }
    let lon = atan2(d.x, d.z);
    let side = select(-0.5, 0.5, params.eye == 1u);
    return side * params.baseline * vec3<f32>(cos(lon), 0.0, -sin(lon));
}

// Where a grid-frame point appears in this eye's rectified picture.
fn seen_from_eye(point: vec3<f32>) -> vec3<f32> {
    if (params.geometry == 1u) {
        return ods_pixel(params.grid, grid_size(), 0.5 * params.baseline, point, params.eye == 0u);
    }
    let viewpoint = select(vec3<f32>(0.0), vec3<f32>(params.baseline, 0.0, 0.0), params.eye == 1u);
    return grid_pixel(params.grid_kind, params.grid, grid_size(), normalize(point - viewpoint));
}

// Put a pixel of the eye's picture on the backdrop, when there is one.
fn backdrop(index: u32, seen_direction: vec3<f32>) {
    if (params.backdrop_range > 0.0) {
        view_points[index] = vec4<f32>(normalize(seen_direction) * params.backdrop_range, BACKDROP);
    }
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.width || id.y >= params.height) { return; }
    let index = id.y * params.width + id.x;
    view_points[index] = vec4<f32>(0.0);
    view_distance[index] = 0.0;

    let uv = (vec2<f32>(id.xy) + vec2<f32>(0.5)) / vec2<f32>(f32(params.width), f32(params.height));
    let seen = unproject_source(params.projection_kind, params.projection_a, params.projection_b, params.projection_c, uv);
    if (seen.w < 0.5) { return; }
    let direction = (transpose(params.eye_from_grid) * vec4<f32>(seen.xyz, 0.0)).xyz;
    let q = grid_pixel(params.grid_kind, params.grid, grid_size(), direction);
    if (q.z < 0.5) {
        backdrop(index, seen.xyz);
        return;
    }

    var found = vec4<f32>(0.0);
    if (params.eye == 0u) {
        let p = q.xy - vec2<f32>(0.5);
        let base = floor(p);
        let f = p - base;
        let x = i32(base.x);
        let y = i32(base.y);
        let c00 = fetch(x, y);
        let c10 = fetch(x + 1, y);
        let c01 = fetch(x, y + 1);
        let c11 = fetch(x + 1, y + 1);
        let all_valid = c00.w > 0.0 && c10.w > 0.0 && c01.w > 0.0 && c11.w > 0.0;
        var agree = false;
        if (all_valid) {
            let i00 = inverse_range(c00);
            let lowest = min(min(i00, inverse_range(c10)), min(inverse_range(c01), inverse_range(c11)));
            let highest = max(max(i00, inverse_range(c10)), max(inverse_range(c01), inverse_range(c11)));
            agree = highest - lowest <= params.inverse_tolerance;
        }
        if (agree) {
            found = mix(mix(c00, c10, f.x), mix(c01, c11, f.x), f.y);
        } else {
            found = fetch(i32(floor(q.x)), i32(floor(q.y)));
        }
    } else {
        let row = i32(floor(q.y));
        let start = i32(floor(q.x)) - 1;
        let centre = eye_centre(direction);
        var nearest = 1e30;
        var have_previous = false;
        var previous = vec4<f32>(0.0);
        var previous_x = 0.0;
        for (var k = 0u; k < params.disparities + 3u; k++) {
            let column = start + i32(k);
            if (params.wraps == 0u && column >= i32(params.grid_width)) { break; }
            let c = fetch(column, row);
            if (c.w <= 0.0) {
                have_previous = false;
                continue;
            }
            let there = seen_from_eye(c.xyz);
            if (there.z < 0.5) {
                have_previous = false;
                continue;
            }
            let offset = apart(there.x, q.x);
            var candidate = vec4<f32>(0.0);
            var hit = false;
            if (have_previous) {
                let before = apart(previous_x, q.x);
                if (before * offset <= 0.0 && abs(inverse_range(previous) - inverse_range(c)) <= params.inverse_tolerance) {
                    let span = offset - before;
                    let t = select(0.5, -before / span, abs(span) > 1e-5);
                    candidate = mix(previous, c, clamp(t, 0.0, 1.0));
                    hit = true;
                }
            }
            if (!hit && abs(offset) <= 0.5) {
                candidate = c;
                hit = true;
            }
            if (hit) {
                let reach = length(candidate.xyz - centre);
                if (reach < nearest) {
                    nearest = reach;
                    found = candidate;
                }
            }
            have_previous = true;
            previous = c;
            previous_x = there.x;
        }
    }
    if (found.w <= 0.0) {
        backdrop(index, seen.xyz);
        return;
    }
    let relative = found.xyz - eye_centre(direction);
    view_distance[index] = length(relative);
    let too_far = 1.0 / max(length(relative), 1e-4) < params.backdrop_inverse;
    let too_unsure = found.w < params.backdrop_confidence;
    if (params.backdrop_range > 0.0 && (too_far || too_unsure)) {
        backdrop(index, seen.xyz);
        return;
    }
    view_points[index] = vec4<f32>((params.eye_from_grid * vec4<f32>(relative, 0.0)).xyz, found.w);
}
