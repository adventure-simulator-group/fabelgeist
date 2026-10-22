// What the forest's branch and leaf shaders share: the instance records the
// botany simulation hands over, and the geometry they are built with.
#define_import_path adventuresim_environment::forest

const TAU: f32 = 6.28318530718;

// A species' colours, sRGB. leaf.w: foliage kind -- 0 broadleaf, 1 needles,
// 2 fronds.
struct Species {
    bark: vec4<f32>,
    leaf: vec4<f32>,
};

// A plant's bounding sphere and species.
struct PlantInfo {
    sphere: vec4<f32>,
    kind: u32,
    padding_x: u32,
    padding_y: u32,
    padding_z: u32,
};

// A branch segment: a tube from ring `a` to ring `b` (centre, radius),
// leaving the one along tangent `ta` and arriving at the other along `tb`.
struct Branch {
    a: vec4<f32>,
    b: vec4<f32>,
    ta: u32,
    tb: u32,
    // The surface's `v` at ring `a`: along the axis, in circumferences.
    v_start: f32,
    light: f32,
    plant: u32,
    flags: u32,
    // The light at ring `a`; `light` is at ring `b`.
    light_start: f32,
    // The surface's `v` at ring `b`.
    v_end: f32,
    axis: u32,
    exit: f32,
    padding_x: u32,
    padding_y: u32,
};

// One leaf cluster at a shoot tip.
struct Leaf {
    center_size: vec4<f32>,
    axis_light: vec4<f32>,
    // xyz: the middle of the plant's crown. w: its radius.
    crown: vec4<f32>,
    kind: u32,
    seed: f32,
    plant: u32,
    pad: u32,
};

// Surface response before world-space PBR lighting; source RGB is linear.
struct SurfaceResponse {
    albedo: vec3<f32>,
    occlusion: f32,
    roughness: f32,
};

// An orthonormal pair around unit `n` that turns smoothly as `n` does, so a
// tube's rings do not twist between tangents; singular only straight down,
// where no branch grows (Frisvad, axes cycled).
fn basis(n: vec3<f32>) -> mat2x3<f32> {
    let m = vec3<f32>(n.z, n.x, n.y);
    let a = 1.0 / max(1.0 + m.z, 1e-6);
    let b = -m.x * m.y * a;
    let f1 = vec3<f32>(1.0 - m.x * m.x * a, b, -m.x);
    let f2 = vec3<f32>(b, 1.0 - m.y * m.y * a, -m.y);
    return mat2x3<f32>(vec3<f32>(f1.y, f1.z, f1.x), vec3<f32>(f2.y, f2.z, f2.x));
}

// A unit vector from its octahedral encoding in two snorm16s.
fn octahedral(packed: u32) -> vec3<f32> {
    let e = unpack2x16snorm(packed);
    var v = vec3<f32>(e.x, e.y, 1.0 - abs(e.x) - abs(e.y));
    if (v.z < 0.0) {
        let s = select(vec2<f32>(-1.0), vec2<f32>(1.0), v.xy >= vec2<f32>(0.0));
        v = vec3<f32>(vec2<f32>(1.0 - abs(v.y), 1.0 - abs(v.x)) * s, v.z);
    }
    return normalize(v);
}

fn linear(c: vec3<f32>) -> vec3<f32> {
    return pow(c, vec3<f32>(2.2));
}

fn hash2(x: i32, y: i32) -> f32 {
    var h = bitcast<u32>(x) * 747796405u + bitcast<u32>(y) * 2891336453u + 1u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    h = (h >> 22u) ^ h;
    return f32(h) / 4294967296.0;
}

// Value noise in the plane, repeating every `period` along x: a whole number
// of cells around a tube, so that it closes on itself.
fn bark_noise(p: vec2<f32>, period: f32) -> f32 {
    let i = floor(p);
    let f = p - i;
    let w = f * f * (3.0 - 2.0 * f);
    let n = i32(period);
    let x0 = ((i32(i.x) % n) + n) % n;
    let x1 = (x0 + 1) % n;
    let y0 = i32(i.y);
    return mix(
        mix(hash2(x0, y0), hash2(x1, y0), w.x),
        mix(hash2(x0, y0 + 1), hash2(x1, y0 + 1), w.x),
        w.y,
    );
}

// A plant turns into its impostor over a band of sizes, this share of the
// switch either side of it, the two dissolving into each other pixel by
// pixel rather than one popping into the other (as SpeedTree's fizzle LOD,
// GPU Gems 3, ch. 4).
const SWITCH_BAND: f32 = 0.3;

// How far a plant spanning `pixels` of radius has turned into its impostor
// at `switch`: zero whole, one an impostor.
fn switch_fade(pixels: f32, switch: f32) -> f32 {
    if switch <= 0.0 {
        return 0.0;
    }
    return clamp((switch * (1.0 + SWITCH_BAND) - pixels) / (2.0 * SWITCH_BAND * switch), 0.0, 1.0);
}

// A number per pixel, zero to one, the same for a plant and its impostor:
// the plant keeps a pixel where it is at least the fade, the impostor
// where it is under (interleaved gradient noise, Jimenez 2014).
fn fizzle(fragment: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(dot(floor(fragment), vec2<f32>(0.06711056, 0.00583715))));
}
