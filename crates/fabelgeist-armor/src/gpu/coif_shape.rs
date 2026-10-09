//! The mail coif's carrier on the device, evaluated per vertex from the
//! wearer's measured drape.
//!
//! Hood and neck vertices carry the sine and cosine of their angle around the
//! head, computed with the design on the host; a coordinate's fourth float
//! names its kind. The neck tube and the flaps hang from rings the carrier
//! also contains, which each vertex re-evaluates rather than reads back.

pub(crate) const SHAPE: &str = r#"
const GAP: u32 = 0u;
const CROWN_HEIGHT: u32 = 1u;
const FRONT_FLAP_LENGTH: u32 = 2u;
const BACK_FLAP_LENGTH: u32 = 3u;
const FLAP_WIDTH: u32 = 4u;

// The wearer's drape follows the frame in the fit buffer: the neck boundary,
// then each flap's sections from the hem up, as height, centre and edge depth.
const DRAPE: u32 = 16u;
const FRONT_HEIGHT: u32 = 0u;
const SIDE_HEIGHT: u32 = 1u;
const BACK_HEIGHT: u32 = 2u;
const NECK_HALF_WIDTH: u32 = 3u;
const CENTER_DEPTH: u32 = 4u;
const FRONT_DEPTH: u32 = 5u;
const BACK_DEPTH: u32 = 6u;
const FRONT_FLAP: u32 = 7u;
const BACK_FLAP: u32 = 22u;
const SECTIONS: u32 = 5u;

const KIND_HOOD: u32 = 4u;
const KIND_NECK: u32 = 5u;
const KIND_FRONT_FLAP: u32 = 6u;

const HOOD_EXPONENT: f32 = 0.72;
const BROW_HEIGHT: f32 = 0.09;
const DRAPE_CONTACT_ROUNDING_M: f32 = 0.002;
const BACK_FLAP_MORPH_RESERVE_M: f32 = 0.005;
const UNDER_CHIN_MORPH_RESERVE_M: f32 = 0.002;

fn drape(k: u32) -> f32 {
    return frames[DRAPE + k];
}

fn half_height() -> f32 {
    return fit.half_extents.y;
}

fn radii() -> vec3<f32> {
    let gap = design[GAP];
    let head = fit.half_extents;
    return vec3<f32>(head.x + gap, head.y * design[CROWN_HEIGHT] + gap, head.z + gap);
}

fn brow() -> f32 {
    return half_height() * BROW_HEIGHT;
}

// A hood ring `t` of the way from the brow to below the chin.
fn hood_point(s: f32, c: f32, t: f32) -> vec3<f32> {
    let r = radii();
    let brow = brow();
    let end_y = half_height() * (-1.12 + 0.16 * (1.0 - c));
    let front = max(c, 0.0);
    let back = max(-c, 0.0);
    let t2 = pow2(t);
    return vec3<f32>(
        r.x * (1.0 - 0.13 * t2) * s,
        brow + (end_y - brow) * t - UNDER_CHIN_MORPH_RESERVE_M * pow4(front) * t2,
        r.z * (1.0 - 0.35 * t2 * front + 0.05 * t2 * back) * c,
    );
}

// The neck/shoulder junction at an angle around the neck: the drape's side
// height and centre depth, blended toward the front or back measurements.
fn neck_boundary(s: f32, c: f32) -> vec3<f32> {
    var end_height = drape(BACK_HEIGHT);
    var end_depth = drape(BACK_DEPTH);
    if (c >= 0.0) {
        end_height = drape(FRONT_HEIGHT);
        end_depth = drape(FRONT_DEPTH);
    }
    let side = drape(SIDE_HEIGHT);
    let center = drape(CENTER_DEPTH);
    return vec3<f32>(
        drape(NECK_HALF_WIDTH) * s,
        side + (end_height - side) * (c * c),
        center + (end_depth - center) * sqrt(abs(c)),
    );
}

// The neck tube, straight from the hood's last ring to the junction.
fn neck_point(s: f32, c: f32, t: f32) -> vec3<f32> {
    let top = hood_point(s, c, 1.0);
    let bottom = neck_boundary(s, c);
    return top + (bottom - top) * t;
}

fn section_height(flap: u32, j: u32) -> f32 {
    return drape(flap + j * 3u);
}

fn section_depth(flap: u32, j: u32, edge: u32) -> f32 {
    return drape(flap + j * 3u + 1u + edge);
}

fn section_slope(flap: u32, edge: u32, j: u32) -> f32 {
    let low = select(j - 1u, 0u, j == 0u);
    let high = min(j + 1u, SECTIONS - 1u);
    return (section_depth(flap, high, edge) - section_depth(flap, low, edge))
        / (section_height(flap, high) - section_height(flap, low));
}

// A flap's centre or edge depth at a height: a cubic Hermite through its
// sections, held at the end sections outside them.
fn interpolate(flap: u32, edge: u32, height: f32) -> f32 {
    let last = SECTIONS - 1u;
    if (height <= section_height(flap, 0u)) {
        return section_depth(flap, 0u, edge);
    }
    if (height >= section_height(flap, last)) {
        return section_depth(flap, last, edge);
    }
    var i = 0u;
    for (; i < last; i = i + 1u) {
        if (height <= section_height(flap, i + 1u)) {
            break;
        }
    }
    let low = section_height(flap, i);
    let span = section_height(flap, i + 1u) - low;
    let t = (height - low) / span;
    let t2 = pow2(t);
    let t3 = pow3(t);
    return (2.0 * t3 - 3.0 * t2 + 1.0) * section_depth(flap, i, edge)
        + (t3 - 2.0 * t2 + t) * span * section_slope(flap, edge, i)
        + (-2.0 * t3 + 3.0 * t2) * section_depth(flap, i + 1u, edge)
        + (t3 - t2) * span * section_slope(flap, edge, i + 1u);
}

fn flap_depth(flap: u32, height: f32, across: f32) -> f32 {
    let center = interpolate(flap, 0u, height);
    let edge = interpolate(flap, 1u, height);
    return center + (edge - center) * pow2(across);
}

// A pendant flap row, hung from the neck junction: it follows the body's
// sections, but bridges hollows along the chord to its lower contact.
fn flap_point(s: f32, c: f32, t: f32, front: bool) -> vec3<f32> {
    var flap = BACK_FLAP;
    var length = design[BACK_FLAP_LENGTH];
    var sign = -1.0;
    // The flap's first attachment vertex sets its width; its sine is a
    // shell value.
    var edge_sine = params.value1;
    if (front) {
        flap = FRONT_FLAP;
        length = design[FRONT_FLAP_LENGTH];
        sign = 1.0;
        edge_sine = params.value0;
    }
    let p = neck_point(s, c, 1.0);
    let edge_x = abs(neck_point(edge_sine, 0.0, 1.0).x);
    let across = p.x / edge_x;
    let y = p.y - length * t;
    let fitted = flap_depth(flap, y, across);
    let transition = min(t * 3.0, 1.0);
    let blend = pow2(transition) * (3.0 - 2.0 * transition);
    let body_profile = p.z + (fitted - p.z) * blend;
    let lower = flap_depth(flap, p.y - length, across);
    let chord = p.z + (lower - p.z) * t;
    let a = sign * body_profile;
    let b = sign * chord;
    let rounding = DRAPE_CONTACT_ROUNDING_M * sin(PI * t);
    var reserve = 0.0;
    if (!front) {
        reserve = BACK_FLAP_MORPH_RESERVE_M * pow2(sin(PI * t)) * max(1.0 - pow2(across), 0.0);
    }
    let z = sign * (a + b + sqrt(pow2(a - b) + pow2(rounding))) * 0.5 - reserve;
    return vec3<f32>(p.x * (1.0 + (design[FLAP_WIDTH] - 1.0) * t), y, z);
}

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    let kind = u32(coord.w);
    if (kind == KIND_HOOD) {
        return ShellVertex(hood_point(coord.x, coord.y, coord.z), 0.0);
    }
    if (kind == KIND_NECK) {
        return ShellVertex(neck_point(coord.x, coord.y, coord.z), 0.0);
    }
    if (kind >= KIND_FRONT_FLAP) {
        return ShellVertex(flap_point(coord.x, coord.y, coord.z, kind == KIND_FRONT_FLAP), 0.0);
    }
    let d = Dome(radii(), brow(), HOOD_EXPONENT, 0.0, 0.0, false, 0u);
    return dome_vertex(d, coord);
}

fn shell_pass(index: u32, coord: vec4<f32>) -> u32 {
    return 0u;
}

fn shell_origin() -> vec3<f32> {
    return vec3<f32>(0.0);
}

fn shell_hinge() -> array<vec3<f32>, 2> {
    return array<vec3<f32>, 2>(vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0));
}
"#;
