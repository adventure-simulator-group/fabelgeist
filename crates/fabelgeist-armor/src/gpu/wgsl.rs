//! WGSL shared by the armor kernels.
//!
//! Points travel as tightly packed `f32` triples, so every buffer of points
//! is an `array<f32>` read and written through these helpers.

/// Constants and scalar helpers: interpolation, smoothsteps and small powers.
pub const MATH: &str = r#"
const PI: f32 = 3.14159265358979;
const TAU: f32 = 6.28318530717959;
const FRAC_PI_2: f32 = 1.57079632679490;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return a + (b - a) * t;
}

fn smoothstep_clamped(t: f32) -> f32 {
    let c = clamp(t, 0.0, 1.0);
    return c * c * (3.0 - 2.0 * c);
}

// The smoothstep polynomial without the clamp, for callers whose argument is
// already in range, such as the footwear profiles.
fn smoothstep_unclamped(t: f32) -> f32 {
    return t * t * (3.0 - 2.0 * t);
}

// Small integer powers by repeated multiplication, exact for small exponents.
fn pow2(x: f32) -> f32 { return x * x; }
fn pow3(x: f32) -> f32 { return x * x * x; }
fn pow4(x: f32) -> f32 { let s = x * x; return s * s; }
fn pow8(x: f32) -> f32 { let s = pow4(x); return s * s; }

fn is_finite(x: f32) -> bool {
    return abs(x) <= 3.4e38;
}
"#;

/// A part frame as the device stores it: origin, three axes, half extents,
/// fifteen floats at `offset` in a frame buffer.
pub const FRAME: &str = r#"
struct Frame {
    origin: vec3<f32>,
    x: vec3<f32>,
    y: vec3<f32>,
    z: vec3<f32>,
    half_extents: vec3<f32>,
};

fn frame_at(offset: u32) -> Frame {
    var v: array<f32, 15>;
    for (var i = 0u; i < 15u; i = i + 1u) {
        v[i] = frames[offset + i];
    }
    return Frame(
        vec3<f32>(v[0], v[1], v[2]),
        vec3<f32>(v[3], v[4], v[5]),
        vec3<f32>(v[6], v[7], v[8]),
        vec3<f32>(v[9], v[10], v[11]),
        vec3<f32>(v[12], v[13], v[14]),
    );
}

// `PartFrame::point`: a local point placed by the frame, summed in the same order.
fn frame_point(f: Frame, local: vec3<f32>) -> vec3<f32> {
    return f.origin + (f.x * local.x + f.y * local.y) + f.z * local.z;
}

fn frame_vector(f: Frame, local: vec3<f32>) -> vec3<f32> {
    return (f.x * local.x + f.y * local.y) + f.z * local.z;
}

fn frame_local(f: Frame, p: vec3<f32>) -> vec3<f32> {
    let d = p - f.origin;
    return vec3<f32>(dot(d, f.x), dot(d, f.y), dot(d, f.z));
}
"#;

/// Indexed reads and writes of packed point buffers.
pub fn points(name: &str) -> String {
    format!(
        r#"
fn {name}_at(i: u32) -> vec3<f32> {{
    return vec3<f32>({name}[i * 3u], {name}[i * 3u + 1u], {name}[i * 3u + 2u]);
}}

fn {name}_set(i: u32, p: vec3<f32>) {{
    {name}[i * 3u] = p.x;
    {name}[i * 3u + 1u] = p.y;
    {name}[i * 3u + 2u] = p.z;
}}
"#
    )
}

/// Readers for a buffer of points that is bound read-only.
pub fn read_points(name: &str) -> String {
    format!(
        r#"
fn {name}_at(i: u32) -> vec3<f32> {{
    return vec3<f32>({name}[i * 3u], {name}[i * 3u + 1u], {name}[i * 3u + 2u]);
}}
"#
    )
}

/// Status bits a kernel raises for the host to turn into an error.
pub const STATUS: &str = r#"
const STATUS_INVALID_SURFACE: u32 = 1u;
const STATUS_DEGENERATE: u32 = 2u;

fn fail(bit: u32) {
    atomicOr(&status[0], bit);
}
"#;

/// The uniform of a pass over `count` items, padded to sixteen bytes.
pub const COUNTED: &str = r#"
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
"#;

/// Floats ordered as unsigned integers, so that atomic minimum and maximum
/// over `u32` give the minimum and maximum of the floats -- exactly, and in
/// any order.
pub const ORDERED_FLOAT: &str = r#"
fn ordered_from_float(value: f32) -> u32 {
    let bits = bitcast<u32>(value);
    if ((bits & 0x80000000u) != 0u) {
        return ~bits;
    }
    return bits | 0x80000000u;
}

fn float_from_ordered(ordered: u32) -> f32 {
    if ((ordered & 0x80000000u) != 0u) {
        return bitcast<f32>(ordered & 0x7fffffffu);
    }
    return bitcast<f32>(~ordered);
}

// The ordered encodings of the widest bounds, for clearing before a reduction.
const ORDERED_POSITIVE_INFINITY: u32 = 0xff800000u;
const ORDERED_NEGATIVE_INFINITY: u32 = 0x007fffffu;
"#;
