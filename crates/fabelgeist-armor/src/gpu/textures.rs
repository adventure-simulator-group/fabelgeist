//! The metal's maps, baked per texel on the device.
//!
//! Three stages: the scratches are stamped into a height field, the
//! engraving (if any) is resampled onto the tile, and every texel then
//! derives its normal, roughness and depth. Every random number comes from
//! one seeded sequence; each scratch and texel jumps straight to its own
//! draws, so the result does not depend on scheduling. A texel stamped by
//! several scratches keeps the deepest groove through an atomic maximum over
//! the groove's bits, which is exact in any order.
//!
//! Every intermediate is fenced so that it rounds the same on any device
//! (see [`fabelgeist_compute::host_float`]).

use super::wgsl;
use crate::material::Metal;

/// Draws each scratch takes: position, angle, length, depth.
pub(super) const DRAWS_PER_SCRATCH: u32 = 5;
/// Invocations per scratch workgroup, along the scratch.
pub(super) const STAMP_GROUP: u32 = 64;

/// Stamp every scratch into the height field.
pub(super) fn scratches_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read_write> height: array<atomic<u32>>;

struct Params {{
    size: u32,
    seed: u32,
    zero: u32,
    pad1: u32,
    scratch_length: f32,
    scratch_width: f32,
    scratch_depth: f32,
    scratch_angle: f32,
    scratch_spread: f32,
    pad2: f32,
    pad3: f32,
    pad4: f32,
}};
@group(0) @binding(1) var<uniform> params: Params;

{math}
{random}

const DRAWS_PER_SCRATCH: u32 = {draws}u;
// A stamp reaches this many texels to each side of the scratch's path.
const STAMP_REACH: i32 = 3;

fn wrap(i: i32, n: i32) -> u32 {{
    return u32(((i % n) + n) % n);
}}

@compute @workgroup_size({group}, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let stamp = id.x;
    let size = f32(params.size);
    var state = lcg_skip(params.seed, id.y * DRAWS_PER_SCRATCH);
    state = lcg_next(state);
    let x = host_mul(draw(state), size);
    state = lcg_next(state);
    let y = host_mul(draw(state), size);
    state = lcg_next(state);
    let swing = host_mul(host_mul(host_fence(draw(state) - 0.5), params.scratch_spread), 2.0);
    let angle = host_fence(params.scratch_angle + swing);
    state = lcg_next(state);
    let reach = host_fence(0.3 + host_mul(draw(state), 0.7));
    let span = host_mul(host_mul(params.scratch_length, size), reach);
    state = lcg_next(state);
    let depth = host_mul(params.scratch_depth, host_fence(0.4 + host_mul(draw(state), 0.6)));
    let steps = u32(ceil(host_mul(span, 2.0)));
    if (stamp > steps) {{
        return;
    }}
    let t = host_div(f32(stamp), f32(max(steps, 1u)));
    let direction = host_sin_cos(angle);
    let px = host_fence(x + host_mul(host_mul(direction.y, span), t));
    let py = host_fence(y + host_mul(host_mul(direction.x, span), t));
    let fade = host_sin(host_mul(PI, t));
    let n = i32(params.size);
    for (var dy = -STAMP_REACH; dy <= STAMP_REACH; dy = dy + 1) {{
        for (var dx = -STAMP_REACH; dx <= STAMP_REACH; dx = dx + 1) {{
            let ix = i32(floor(px)) + dx;
            let iy = i32(floor(py)) + dy;
            let ox = host_fence(f32(ix) - px);
            let oy = host_fence(f32(iy) - py);
            let apart = host_sqrt(host_fence(host_mul(ox, ox) + host_mul(oy, oy)));
            let edge = max(host_fence(1.0 - host_div(apart, params.scratch_width)), 0.0);
            let groove = host_mul(host_mul(edge, depth), fade);
            // Grooves are compared by their bits, which order positive floats;
            // anything not above the untouched zero leaves the texel alone.
            if (groove > 0.0) {{
                atomicMax(&height[wrap(iy, n) * params.size + wrap(ix, n)], bitcast<u32>(groove));
            }}
        }}
    }}
}}
"#,
        math = wgsl::math(),
        random = wgsl::RANDOM,
        draws = DRAWS_PER_SCRATCH,
        group = STAMP_GROUP,
    )
}

/// Resample the relief image onto the tile: the recess of a height map, or
/// the turned slopes of a normal map.
pub(super) fn engraving_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> image: array<f32>;
@group(0) @binding(1) var<storage, read_write> recess: array<f32>;
@group(0) @binding(2) var<storage, read_write> slopes: array<f32>;

struct Params {{
    size: u32,
    image_width: u32,
    image_height: u32,
    normal_map: u32,
    tiles: f32,
    sin_rotation: f32,
    cos_rotation: f32,
    strength: f32,
    zero: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;

{math}

fn wrap(i: i32, n: u32) -> u32 {{
    let m = i32(n);
    return u32(((i % m) + m) % m);
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let index = id.x;
    let n = params.size;
    if (index >= n * n) {{
        return;
    }}
    let size = f32(n);
    let px = host_fence(host_div(f32(index % n) + 0.5, size) - 0.5);
    let py = host_fence(host_div(f32(index / n) + 0.5, size) - 0.5);
    let s = params.sin_rotation;
    let c = params.cos_rotation;
    // Turn the tile back into the image, then repeat the image.
    let qx = host_fence(host_mul(c, px) + host_mul(s, py));
    let qy = host_fence(host_mul(-s, px) + host_mul(c, py));
    let width = f32(params.image_width);
    let height = f32(params.image_height);
    let x = host_fence(host_mul(host_fence(host_mul(qx, params.tiles) + 0.5), width) - 0.5);
    let y = host_fence(host_mul(host_fence(host_mul(qy, params.tiles) + 0.5), height) - 0.5);
    let x0 = floor(x);
    let y0 = floor(y);
    let fx = host_fence(x - x0);
    let fy = host_fence(y - y0);
    let c0 = wrap(i32(x0), params.image_width);
    let c1 = wrap(i32(x0) + 1, params.image_width);
    let r0 = wrap(i32(y0), params.image_height) * params.image_width;
    let r1 = wrap(i32(y0) + 1, params.image_height) * params.image_width;
    let w00 = host_mul(host_fence(1.0 - fx), host_fence(1.0 - fy));
    let w10 = host_mul(fx, host_fence(1.0 - fy));
    let w01 = host_mul(host_fence(1.0 - fx), fy);
    let w11 = host_mul(fx, fy);
    if (params.normal_map == 0u) {{
        var sum = host_mul(image[r0 + c0], w00);
        sum = host_fence(sum + host_mul(image[r0 + c1], w10));
        sum = host_fence(sum + host_mul(image[r1 + c0], w01));
        sum = host_fence(sum + host_mul(image[r1 + c1], w11));
        recess[index] = host_fence(1.0 - sum);
        return;
    }}
    var slope: vec2<f32>;
    for (var k = 0u; k < 2u; k = k + 1u) {{
        var sum = host_mul(image[(r0 + c0) * 2u + k], w00);
        sum = host_fence(sum + host_mul(image[(r0 + c1) * 2u + k], w10));
        sum = host_fence(sum + host_mul(image[(r1 + c0) * 2u + k], w01));
        sum = host_fence(sum + host_mul(image[(r1 + c1) * 2u + k], w11));
        slope[k] = sum;
    }}
    // The image turned with the tile, so its slopes turn too.
    let sx = host_fence(host_mul(c, slope.x) - host_mul(s, slope.y));
    let sy = host_fence(host_mul(s, slope.x) + host_mul(c, slope.y));
    slopes[index * 2u] = host_mul(sx, params.strength);
    slopes[index * 2u + 1u] = host_mul(sy, params.strength);
}}
"#,
        math = wgsl::math(),
    )
}

/// The slopes of a height map's cut, by central differences.
pub(super) fn cut_slopes_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> recess: array<f32>;
@group(0) @binding(1) var<storage, read_write> slopes: array<f32>;

struct Params {{
    size: u32,
    zero: u32,
    pad1: u32,
    pad2: u32,
    depth: f32,
    pad3: f32,
    pad4: f32,
    pad5: f32,
}};
@group(0) @binding(2) var<uniform> params: Params;

{math}

const TILES_PER_METRE: f32 = {tiles_per_metre:?};

fn cut(x: u32, y: u32) -> f32 {{
    let n = params.size;
    return host_mul(-params.depth, recess[(y % n) * n + x % n]);
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let index = id.x;
    let n = params.size;
    if (index >= n * n) {{
        return;
    }}
    let x = index % n;
    let y = index / n;
    let texel = host_div(1.0, host_mul(TILES_PER_METRE, f32(n)));
    let across = host_mul(2.0, texel);
    slopes[index * 2u] = host_div(host_fence(cut(x + 1u, y) - cut(x + n - 1u, y)), across);
    slopes[index * 2u + 1u] = host_div(host_fence(cut(x, y + 1u) - cut(x, y + n - 1u)), across);
}}
"#,
        math = wgsl::math(),
        tiles_per_metre = Metal::TILES_PER_METRE,
    )
}

/// Every texel's normal, roughness and depth, as RGBA8 words.
pub(super) fn bake_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> height: array<u32>;
@group(0) @binding(1) var<storage, read> slopes: array<f32>;
@group(0) @binding(2) var<storage, read> recess: array<f32>;
@group(0) @binding(3) var<storage, read_write> normal: array<u32>;
@group(0) @binding(4) var<storage, read_write> metal_roughness: array<u32>;
@group(0) @binding(5) var<storage, read_write> depth: array<u32>;

struct Params {{
    size: u32,
    seed: u32,
    scratch_draws: u32,
    engraved: u32,
    roughness: f32,
    recess_roughness: f32,
    zero: u32,
    pad0: u32,
}};
@group(0) @binding(6) var<uniform> params: Params;

{math}
{random}

const SCRATCH_ROUGHNESS: f32 = {scratch_roughness:?};
// Amplitude of the per-texel roughness noise.
const ROUGHNESS_NOISE: f32 = 0.025;

fn h(x: u32, y: u32) -> f32 {{
    return bitcast<f32>(height[y * params.size + x]);
}}

// `(v * 255.0) as u8`: truncated and saturated.
fn channel(v: f32) -> u32 {{
    return u32(clamp(host_mul(v, 255.0), 0.0, 255.0));
}}

// `v / length * 0.5 + 0.5`: a normal component as a byte.
fn tilt(v: f32, length: f32) -> u32 {{
    return channel(host_fence(host_mul(host_div(v, length), 0.5) + 0.5));
}}

fn rgba(r: u32, g: u32, b: u32, a: u32) -> u32 {{
    return r | (g << 8u) | (b << 16u) | (a << 24u);
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let index = id.x;
    let n = params.size;
    if (index >= n * n) {{
        return;
    }}
    let x = index % n;
    let y = index / n;
    let here = h(x, y);
    var dx = host_mul(host_fence(h((x + 1u) % n, y) - h((x + n - 1u) % n, y)), 2.0);
    var dy = host_mul(host_fence(h(x, (y + 1u) % n) - h(x, (y + n - 1u) % n)), 2.0);
    var cut = 0.0;
    var cut_roughness = 0.0;
    if (params.engraved != 0u) {{
        dx = host_fence(dx + slopes[index * 2u]);
        dy = host_fence(dy + slopes[index * 2u + 1u]);
        cut = recess[index];
        cut_roughness = params.recess_roughness;
    }}
    // glTF tangent space: +X right, +Y up the image, so a surface rising to
    // the right or down the image tilts the normal away.
    let norm = host_sqrt(host_fence(host_fence(host_mul(dx, dx) + host_mul(dy, dy)) + 1.0));
    normal[index] = rgba(tilt(-dx, norm), tilt(dy, norm), tilt(1.0, norm), 255u);
    let noise = draw(lcg_skip(params.seed, params.scratch_draws + index + 1u));
    var roughness = host_fence(params.roughness + host_mul(here, SCRATCH_ROUGHNESS));
    roughness = host_fence(roughness + host_mul(cut, cut_roughness));
    roughness = host_fence(roughness + host_mul(host_fence(noise - 0.5), ROUGHNESS_NOISE));
    metal_roughness[index] = rgba(255u, channel(clamp(roughness, 0.0, 1.0)), 255u, 255u);
    let c = channel(cut);
    depth[index] = rgba(c, c, c, 255u);
}}
"#,
        math = wgsl::math(),
        random = wgsl::RANDOM,
        scratch_roughness = Metal::SCRATCH_ROUGHNESS,
    )
}
