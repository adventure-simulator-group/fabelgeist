//! WGSL shared by the grooming kernels: the host's hashes and noise, the
//! maths of strand frames, the head's distance field, and a walker that
//! samples a strand by arc length. Pasted into each kernel after its own
//! bindings, as the compute kernels compose theirs.

/// Hashes, as [`crate::rng`] computes them, bit for bit.
pub const RANDOM: &str = r#"
fn hash2(a: u32, b: u32) -> u32 {
    var h = (a * 0x9E3779B1u) ^ (((b << 16u) | (b >> 16u)) * 0x85EBCA6Bu);
    h = h ^ (h >> 16u);
    h = h * 0x85EBCA6Bu;
    h = h ^ (h >> 13u);
    h = h * 0xC2B2AE35u;
    return h ^ (h >> 16u);
}

fn hash3(a: u32, b: u32, c: u32) -> u32 {
    return hash2(hash2(a, b), c);
}

// Uniform in [0, 1) for an id under a seed.
fn unit(id: u32, seed: u32) -> f32 {
    return f32(hash2(id, seed) >> 8u) / 16777216.0;
}

fn signed1(id: u32, seed: u32) -> f32 {
    return unit(id, seed) * 2.0 - 1.0;
}

fn signed3(id: u32, seed: u32) -> vec3<f32> {
    return vec3<f32>(
        signed1(id, hash2(seed, 1u)),
        signed1(id, hash2(seed, 2u)),
        signed1(id, hash2(seed, 3u)),
    );
}
"#;

/// Blender's noise, as [`crate::noise`] computes it.
pub const NOISE: &str = r#"
fn noise_fade(t: f32) -> f32 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn noise_gradient(ix: i32, iy: i32, iz: i32, x: f32, y: f32, z: f32) -> f32 {
    let h = hash3(bitcast<u32>(ix), bitcast<u32>(iy), bitcast<u32>(iz)) % 12u;
    var u = y;
    var v = z;
    if (h <= 3u) {
        u = x;
        v = y;
    } else if (h <= 7u) {
        u = x;
        v = z;
    }
    if ((h & 1u) != 0u) {
        u = -u;
    }
    if ((h & 2u) != 0u) {
        v = -v;
    }
    return u + v;
}

fn perlin(p: vec3<f32>) -> f32 {
    let f = floor(p);
    let i = vec3<i32>(f);
    let q = p - f;
    let w = vec3<f32>(noise_fade(q.x), noise_fade(q.y), noise_fade(q.z));
    let g000 = noise_gradient(i.x, i.y, i.z, q.x, q.y, q.z);
    let g100 = noise_gradient(i.x + 1, i.y, i.z, q.x - 1.0, q.y, q.z);
    let g010 = noise_gradient(i.x, i.y + 1, i.z, q.x, q.y - 1.0, q.z);
    let g110 = noise_gradient(i.x + 1, i.y + 1, i.z, q.x - 1.0, q.y - 1.0, q.z);
    let g001 = noise_gradient(i.x, i.y, i.z + 1, q.x, q.y, q.z - 1.0);
    let g101 = noise_gradient(i.x + 1, i.y, i.z + 1, q.x - 1.0, q.y, q.z - 1.0);
    let g011 = noise_gradient(i.x, i.y + 1, i.z + 1, q.x, q.y - 1.0, q.z - 1.0);
    let g111 = noise_gradient(i.x + 1, i.y + 1, i.z + 1, q.x - 1.0, q.y - 1.0, q.z - 1.0);
    let value = mix(
        mix(mix(g000, g100, w.x), mix(g010, g110, w.x), w.y),
        mix(mix(g001, g101, w.x), mix(g011, g111, w.x), w.y),
        w.z,
    );
    return clamp(value * 0.982, -1.0, 1.0);
}

// Octaves at twice the frequency, `roughness` the amplitude, in [0, 1].
fn fractal(p: vec3<f32>, octaves: u32, roughness: f32) -> f32 {
    var sum = 0.0;
    var amplitude = 1.0;
    var total = 0.0;
    var q = p;
    for (var o = 0u; o < max(octaves, 1u); o++) {
        sum += perlin(q) * amplitude;
        total += amplitude;
        amplitude *= roughness;
        q *= 2.0;
    }
    return 0.5 + 0.5 * sum / total;
}

fn noise_color(p: vec3<f32>, scale: f32) -> vec3<f32> {
    let q = p * scale;
    return vec3<f32>(
        fractal(q, 1u, 0.5),
        fractal(q + vec3<f32>(31.416, -47.853, 12.679), 1u, 0.5),
        fractal(q + vec3<f32>(-83.1, 71.3, -21.7), 1u, 0.5),
    );
}

fn white(p: vec3<f32>) -> f32 {
    let h = hash3(bitcast<u32>(p.x), bitcast<u32>(p.y), hash2(bitcast<u32>(p.z), 0x5bd1e995u));
    return f32(h >> 8u) / 16777216.0;
}
"#;

/// Frames, rotations, falloffs: [`crate::strands`] and [`crate::ops`].
pub const MATH: &str = r#"
const PI: f32 = 3.14159265;
const TAU: f32 = 6.2831853;

fn smoothstep1(e0: f32, e1: f32, x: f32) -> f32 {
    if (abs(e1 - e0) <= 1e-12) {
        return 0.0;
    }
    let t = clamp((x - e0) / (e1 - e0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn smootherstep1(e0: f32, e1: f32, x: f32) -> f32 {
    if (abs(e1 - e0) <= 1e-12) {
        return 0.0;
    }
    let t = clamp((x - e0) / (e1 - e0), 0.0, 1.0);
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

// Blender's `shape_range` group with base 2.
fn shape_range(value: f32, lo: f32, hi: f32, shape: f32) -> f32 {
    if (abs(shape) < 1e-5) {
        return 1.0;
    }
    var x = 0.0;
    if (abs(hi - lo) > 1e-12) {
        x = (value - lo) / (hi - lo);
    }
    let b = 2.0;
    let a = min(abs(shape), 0.999999);
    let negative = shape < 0.0;
    let flip = select(negative, !negative, a > 0.5);
    let xx = select(x, 1.0 - x, flip);
    let pingpong = select(1.0 - a, a, a <= 0.5);
    let m = pingpong * 2.0 * (1.0 - 1.0 / b);
    let exponent = -log(max(1.0 - m, 1e-5)) / log(b);
    var y = pow(max(xx, 0.0), exponent);
    if (a > 0.5) {
        y = 1.0 - y;
    }
    return clamp(lo + clamp(y, 0.0, 1.0) * (hi - lo), min(lo, hi), max(lo, hi));
}

fn safe_normalize(v: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let l = length(v);
    if (l > 1e-12 && l < 1e30) {
        return v / l;
    }
    return fallback;
}

// Blender's `math::orthogonal`.
fn orthogonal(v: vec3<f32>) -> vec3<f32> {
    let a = abs(v);
    var o = vec3<f32>(-v.y, v.x, 0.0);
    if (a.x <= a.y && a.x <= a.z) {
        o = vec3<f32>(0.0, -v.z, v.y);
    } else if (a.y <= a.z) {
        o = vec3<f32>(-v.z, 0.0, v.x);
    }
    return safe_normalize(o, vec3<f32>(1.0, 0.0, 0.0));
}

fn rotate(v: vec3<f32>, axis: vec3<f32>, angle: f32) -> vec3<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return v * c + cross(axis, v) * s + axis * (dot(axis, v) * (1.0 - c));
}

fn transport(v: vec3<f32>, source: vec3<f32>, target_dir: vec3<f32>) -> vec3<f32> {
    let axis = cross(source, target_dir);
    let s = length(axis);
    let c = dot(source, target_dir);
    if (s < 1e-7) {
        return select(-v, v, c > 0.0);
    }
    return rotate(v, axis / s, atan2(s, c));
}

// `Vec3::lerp` as the host computes it: `mix` may round otherwise.
fn lerp3(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    return a + (b - a) * t;
}

fn in_plane(v: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    return v - n * dot(v, n);
}
"#;

/// The mannequin's distance field, from `head_shape` (see
/// [`super::super::head_shape`] for the layout).
pub const HEAD: &str = r#"
fn ellipsoid_distance(p: vec3<f32>, center: vec3<f32>, radii: vec3<f32>) -> f32 {
    let q = p - center;
    let k0 = length(q / radii);
    let k1 = length(q / (radii * radii));
    if (k1 < 1e-12) {
        return -min(radii.x, min(radii.y, radii.z));
    }
    return k0 * (k0 - 1.0) / k1;
}

fn capsule_distance(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, radius: f32) -> f32 {
    let ab = b - a;
    let t = clamp(dot(p - a, ab) / dot(ab, ab), 0.0, 1.0);
    return length(p - (a + ab * t)) - radius;
}

fn smooth_min(a: f32, b: f32, k: f32) -> f32 {
    let h = max(k - abs(a - b), 0.0) / k;
    return min(a, b) - h * h * k * 0.25;
}

fn head_part(i: u32, p: vec3<f32>) -> f32 {
    return ellipsoid_distance(p, head_shape[i].xyz, head_shape[i + 1u].xyz);
}

fn head_distance(p: vec3<f32>) -> f32 {
    var d = smooth_min(head_part(0u, p), head_part(2u, p), 0.04);
    d = smooth_min(d, head_part(4u, p), 0.012);
    let ears = min(head_part(6u, p), head_part(8u, p));
    d = smooth_min(d, ears, 0.006);
    let neck = smooth_min(d, capsule_distance(p, head_shape[10].xyz, head_shape[11].xyz, head_shape[10].w), 0.03);
    let torso = smooth_min(head_part(12u, p), head_part(14u, p), 0.08);
    let body = smooth_min(neck, torso, 0.06);
    return max(body, head_shape[16].x - p.y);
}

fn head_gradient(p: vec3<f32>) -> vec3<f32> {
    let e = 2e-4;
    let g = vec3<f32>(
        head_distance(p + vec3<f32>(e, 0.0, 0.0)) - head_distance(p - vec3<f32>(e, 0.0, 0.0)),
        head_distance(p + vec3<f32>(0.0, e, 0.0)) - head_distance(p - vec3<f32>(0.0, e, 0.0)),
        head_distance(p + vec3<f32>(0.0, 0.0, e)) - head_distance(p - vec3<f32>(0.0, 0.0, e)),
    );
    return safe_normalize(g, vec3<f32>(0.0, 1.0, 0.0));
}

// Out until at least `offset` off the body, as `collider::push_out` does.
fn push_out(q: vec3<f32>, offset: f32) -> vec3<f32> {
    var p = q;
    for (var k = 0; k < 6; k++) {
        let d = head_distance(p);
        if (d >= offset) {
            break;
        }
        p += head_gradient(p) * (offset - d + 1e-5);
    }
    return p;
}
"#;

/// A strand's tangents (Blender's poly-curve tangent) and arc lengths, and
/// a walker sampling a strand by arc length with parallel-transported
/// normals -- [`crate::strands::CurveSampler`] computed as it goes, so a
/// kernel sampling a guide needs no frames stored for it. Reads `src`.
pub const WALK: &str = r#"
fn point_at(base: u32, j: u32) -> vec3<f32> {
    return src[base + j].xyz;
}

fn tangent_at(base: u32, points: u32, j: u32) -> vec3<f32> {
    var d: vec3<f32>;
    if (j == 0u) {
        d = point_at(base, 1u) - point_at(base, 0u);
    } else if (j == points - 1u) {
        d = point_at(base, j) - point_at(base, j - 1u);
    } else {
        let a = point_at(base, j + 1u) - point_at(base, j);
        let b = point_at(base, j) - point_at(base, j - 1u);
        d = safe_normalize(a, vec3<f32>(0.0, 1.0, 0.0)) + safe_normalize(b, vec3<f32>(0.0, 1.0, 0.0));
    }
    return safe_normalize(d, vec3<f32>(0.0, 1.0, 0.0));
}

fn strand_length(base: u32, points: u32) -> f32 {
    var total = 0.0;
    for (var j = 1u; j < points; j++) {
        total += distance(point_at(base, j), point_at(base, j - 1u));
    }
    return total;
}

struct Walker {
    base: u32,
    points: u32,
    i: u32,
    l0: f32,
    l1: f32,
    t0: vec3<f32>,
    t1: vec3<f32>,
    n0: vec3<f32>,
    n1: vec3<f32>,
    total: f32,
};

struct Frame {
    position: vec3<f32>,
    tangent: vec3<f32>,
    normal: vec3<f32>,
    binormal: vec3<f32>,
};

fn walker_new(base: u32, points: u32) -> Walker {
    var w: Walker;
    w.base = base;
    w.points = points;
    w.i = 0u;
    w.l0 = 0.0;
    w.t0 = tangent_at(base, points, 0u);
    w.n0 = orthogonal(w.t0);
    w.t1 = tangent_at(base, points, 1u);
    w.n1 = transport(w.n0, w.t0, w.t1);
    w.l1 = distance(point_at(base, 1u), point_at(base, 0u));
    w.total = strand_length(base, points);
    return w;
}

// The frame at arc length `s`, clamped to the strand. Lengths must be
// asked for in increasing order.
fn walker_at(w: ptr<function, Walker>, length_along: f32) -> Frame {
    let s = clamp(length_along, 0.0, (*w).total);
    while ((*w).l1 <= s && (*w).i + 2u < (*w).points) {
        let next = (*w).i + 2u;
        (*w).i += 1u;
        (*w).l0 = (*w).l1;
        (*w).t0 = (*w).t1;
        (*w).n0 = (*w).n1;
        (*w).t1 = tangent_at((*w).base, (*w).points, next);
        (*w).n1 = transport((*w).n0, (*w).t0, (*w).t1);
        (*w).l1 += distance(point_at((*w).base, next), point_at((*w).base, next - 1u));
    }
    let span = (*w).l1 - (*w).l0;
    var t = 0.0;
    if (span > 1e-12) {
        t = clamp((s - (*w).l0) / span, 0.0, 1.0);
    }
    var f: Frame;
    f.position = mix(point_at((*w).base, (*w).i), point_at((*w).base, (*w).i + 1u), t);
    f.tangent = safe_normalize(mix((*w).t0, (*w).t1, t), (*w).t0);
    let n = mix((*w).n0, (*w).n1, t);
    f.normal = safe_normalize(n - f.tangent * dot(n, f.tangent), (*w).n0);
    f.binormal = cross(f.tangent, f.normal);
    return f;
}

// `Restore Curve Segment Length`: `dst` given back `src`'s segment lengths.
fn restore_lengths(base: u32, points: u32, factor: f32) {
    var previous = dst[base].xyz;
    var previous_old = dst[base].xyz;
    for (var j = 1u; j < points; j++) {
        let current = dst[base + j].xyz;
        let direction = current - previous_old;
        let current_length = length(direction);
        let rest_vector = src[base + j].xyz - src[base + j - 1u].xyz;
        let rest = length(rest_vector);
        let target_length = current_length + (rest - current_length) * factor;
        var unit_direction = safe_normalize(rest_vector, vec3<f32>(0.0, -1.0, 0.0));
        if (current_length > 1e-12) {
            unit_direction = direction / current_length;
        }
        previous_old = current;
        let placed = previous + unit_direction * target_length;
        dst[base + j] = vec4<f32>(placed, dst[base + j].w);
        previous = placed;
    }
}

fn copy_strand(base: u32, points: u32) {
    for (var j = 0u; j < points; j++) {
        dst[base + j] = src[base + j];
    }
}
"#;

/// The flat item index of a folded dispatch with 64-wide workgroups.
pub const INDEX: &str = r#"
fn item(id: vec3<u32>, groups: vec3<u32>) -> u32 {
    return id.y * groups.x * 64u + id.x;
}
"#;

/// A strand's per-strand data on the GPU, as [`super::super::StrandGpu`].
pub const STRAND: &str = r#"
struct StrandGpu {
    normal: vec3<f32>,
    id: u32,
    uv: vec2<f32>,
    group: i32,
    radius: f32,
};
"#;
