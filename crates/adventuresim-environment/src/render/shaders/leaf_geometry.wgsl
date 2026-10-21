// Exact grown blade evaluation; no camera, material binding or derivatives.
#define_import_path adventuresim_environment::leaf_geometry
#import adventuresim_environment::forest::{TAU, Leaf, basis}

const GOLDEN: f32 = 2.39996323;

struct LeafVertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    seed: f32,
};

fn rand(seed: f32, i: u32, k: f32) -> f32 {
    return fract(sin(seed * 91.17 + f32(i) * 12.9898 + k * 78.233) * 43758.5453);
}

fn leaf_vertex(inst: Leaf, foliage: u32, i: u32, corner: vec2<f32>, blades: u32) -> LeafVertex {
    // Fewer blades cover as much, each larger.
    let spread = sqrt(10.0 / f32(blades));
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let tip = inst.center_size.xyz;
    let size = inst.center_size.w;
    let axis = normalize(inst.axis_light.xyz);
    let frame = basis(axis);
    let r0 = rand(inst.seed, i, 0.0);
    let r1 = rand(inst.seed, i, 1.0);
    let r2 = rand(inst.seed, i, 2.0);

    var out: LeafVertex;
    var world: vec3<f32>;
    var normal: vec3<f32>;
    var uv = corner;
    if (foliage == 2u) {
        // One frond along `axis`, arching over under its own weight: blade
        // pairs along it, hinged at the rib.
        let side = select(-1.0, 1.0, i % 2u == 1u);
        let segments = f32(max(blades / 2u, 1u));
        let len = size * 4.0;
        let droop = 0.3 + 0.35 * fract(inst.seed * 13.7);
        let t = (f32(i / 2u) + corner.y) / segments;
        let along = normalize(axis - up * (2.0 * droop * t));
        var flat = cross(along, up);
        if (dot(flat, flat) < 1e-4) {
            flat = frame[0];
        }
        let level = normalize(flat);
        let face = normalize(cross(level, along)) * select(-1.0, 1.0, dot(cross(level, along), up) >= 0.0);
        let hinge = 0.5;
        let across = level * side;
        normal = normalize(face * cos(hinge) + across * sin(hinge));
        let spine = tip + (axis * t - up * droop * t * t) * len;
        let half = 0.5 * max(len * 0.3 * sin(3.14159 * min(t * 1.15, 1.0)), len * 0.02);
        let leaflet = across * cos(hinge) - face * sin(hinge);
        world = spine + leaflet * half * corner.x;
        uv = vec2<f32>(0.5 + 0.5 * side * corner.x, t);
    } else {
        // Blades along the shoot's last stretch, the youngest at its tip.
        let needles = foliage == 1u;
        let t = (f32(i) + 0.7 * r0) / f32(blades);
        let attach = tip - axis * size * select(0.9, 1.6, needles) * t;
        let phi = inst.seed * TAU + f32(i) * GOLDEN;
        let radial = cos(phi) * frame[0] + sin(phi) * frame[1];
        let forward = select(0.55, 0.9, needles) + 0.3 * r1;
        let dir = normalize(radial + axis * forward - up * (0.15 + 0.35 * r2));
        var level = cross(up, dir);
        if (dot(level, level) < 1e-4) {
            level = frame[0];
        }
        level = normalize(level);
        let roll = (r0 - 0.5) * select(0.9, 1.4, needles);
        let above = cross(dir, level);
        let across = normalize(level * cos(roll) + above * sin(roll));
        // Match the quad's (across, along) winding. The fragment flips this
        // normal on a back face; an upward normal would invert both sides.
        normal = normalize(cross(across, dir));
        var len = size * (0.62 + 0.3 * r1) * (1.0 - 0.25 * t) * spread;
        var width = len * (0.5 + 0.12 * r2);
        if (needles) {
            len = size * (1.3 + 0.4 * r1) * spread;
            width = len * 0.5;
        }
        world = attach + dir * len * uv.y + across * width * (uv.x - 0.5);
    }

    out.seed = leaf_seed(inst.seed, i, foliage);
    out.position = world;
    out.normal = normal;
    out.uv = uv;
    return out;
}

// The original per-blade pigment seed, also recoverable from an integer address.
fn leaf_seed(seed: f32, i: u32, foliage: u32) -> f32 {
    if foliage == 2u {
        return seed;
    }
    return fract(seed * 7.13 + f32(i) * 0.618);
}
