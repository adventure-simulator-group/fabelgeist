//! The helmet kernel's shape: every carrier vertex of every helmet, from its
//! coordinates, the design floats and the head frame's half extents.

use super::burgonet::CHEEK_SHELL;
use super::codes::{CheekRow, PostMap, Slot, VertexKind};

pub(super) fn source() -> String {
    format!(
        "{slots}const BOWL_TABLE: u32 = {table}u;\n{kinds}{maps}{rows}\
         const CHEEK_SHELL: f32 = {CHEEK_SHELL:?};\n{SHAPE}",
        slots = Slot::wgsl(""),
        table = Slot::COUNT,
        kinds = VertexKind::wgsl("KIND_"),
        maps = PostMap::wgsl("MAP_"),
        rows = CheekRow::wgsl("ROW_"),
    )
}

const SHAPE: &str = r#"
// The bare-head frame extends from chin to crown; the brow line is this far
// above its centre, per unit of half height.
const BROW_HEIGHT: f32 = 0.09;
const AROUND: f32 = 48.0;
const RIDGE_SPREAD: f32 = 0.30;
const CREST_HALF_WIDTH_SHARE: f32 = 0.06;
const CREST_HALF_WIDTH_M: f32 = 0.005;
const CREST_BASE_WIDTH_SHARE: f32 = 0.12;
const CREST_BASE_WIDTH_M: f32 = 0.010;
const BARBUTE_CORNER_SHARE: f32 = 0.45;
const BARBUTE_MINIMUM_CORNER_M: f32 = 1e-6;
const BARBUTE_CORNER_ROWS: f32 = 4.0;
const BARBUTE_LOWER_ROWS: f32 = 8.0;
const NAPE_FLARE_START: f32 = 0.75;
const VISOR_ARM_WIDTH_M: f32 = 0.012;
const SALLET_ORIGINAL_ARC: f32 = 1.04719755119660;

fn shell_origin() -> vec3<f32> {
    return vec3<f32>(params.origin_x, params.origin_y, params.origin_z);
}

fn shell_pass(index: u32, coord: vec4<f32>) -> u32 {
    return 0u;
}

// Only the visor carries a hinge: below the pivot arms' upper edge.
fn shell_hinge() -> array<vec3<f32>, 2> {
    let top = brow() - design[SIGHT_GAP];
    return array<vec3<f32>, 2>(
        vec3<f32>(0.0, top + design[PIVOT_RISE] - VISOR_ARM_WIDTH_M * 0.5, 0.0),
        vec3<f32>(1.0, 0.0, 0.0),
    );
}

fn head_radii() -> vec3<f32> {
    let head = fit.half_extents;
    let gap = design[GAP];
    return vec3<f32>(head.x + gap, head.y * design[CROWN_HEIGHT] + gap, head.z + gap);
}

fn dome_radii() -> vec3<f32> {
    let radii = head_radii();
    return vec3<f32>(radii.x, radii.y, radii.z + design[Z_EXTENSION]);
}

fn brow() -> f32 {
    let height = fit.half_extents.y;
    return height * BROW_HEIGHT + height * design[BROW_RISE];
}

// `pow` extended to a base that may be zero or negative, which gives zero.
fn power(base: f32, exponent: f32) -> f32 {
    if (base <= 0.0) {
        return 0.0;
    }
    return pow(base, exponent);
}

fn flute_fan(u: f32, v: f32) -> f32 {
    let lateral = 2.0 * u - 1.0;
    let span = design[FLUTE_SPREAD];
    let fan = design[FLUTE_LOWER_SPREAD] + (1.0 - design[FLUTE_LOWER_SPREAD]) * smoothstep_clamped(v);
    let absolute = abs(lateral);
    var mapped: f32;
    if (absolute <= span) {
        mapped = absolute * fan;
    } else {
        mapped = span * fan + (absolute - span) * (1.0 - span * fan) / (1.0 - span);
    }
    return (1.0 + sign(lateral) * mapped) * 0.5;
}

fn flute_relief(u: f32, v: f32) -> f32 {
    let count = design[FLUTE_COUNT];
    let pitch = design[FLUTE_SPREAD] / count;
    let start = (1.0 - design[FLUTE_SPREAD]) * 0.5;
    let slot = floor((u - start) / pitch);
    if (slot < 0.0 || slot >= count) {
        return 0.0;
    }
    let center = start + (slot + 0.5) * pitch;
    let distance = abs(u - center) / (pitch * design[FLUTE_WIDTH] * 0.5);
    if (distance >= 1.0) {
        return 0.0;
    }
    let fade = smoothstep_clamped((v - design[FLUTE_START]) / design[FLUTE_FADE])
        * smoothstep_clamped((design[FLUTE_END] - v) / design[FLUTE_FADE]);
    return design[FLUTE_DEPTH] * fade * (1.0 + cos(PI * distance)) * 0.5;
}

fn crest_half_width(radii: vec3<f32>) -> f32 {
    return min(radii.x * CREST_HALF_WIDTH_SHARE, CREST_HALF_WIDTH_M);
}

fn crest_base_width(radii: vec3<f32>) -> f32 {
    return min(radii.x * CREST_BASE_WIDTH_SHARE, CREST_BASE_WIDTH_M);
}

// A crown point at a latitude and an angle over the head: the bowl, its sides
// filled out by the design's fullness, with the crest raised from its middle.
fn crown_point(radii: vec3<f32>, brow: f32, latitude: f32, angle: f32) -> vec3<f32> {
    let y = sin(latitude) * sin(angle);
    let radial = sqrt(max(1.0 - y * y, 0.0));
    var fullness = 1.0;
    if (radial > 1e-6) {
        fullness = pow(radial, design[FULLNESS] - 1.0);
    }
    let x = radii.x * cos(latitude);
    var across = 0.0;
    if (design[CREST] > 0.0) {
        let base = crest_base_width(radii);
        across = clamp((base - abs(x)) / (base - crest_half_width(radii)), 0.0, 1.0);
    }
    let crest_phase = clamp((angle / PI - 0.08) / 0.84, 0.0, 1.0);
    let rise = power(max(sin(PI * crest_phase), 0.0), 0.8);
    let crest = design[CREST] * across * rise;
    return vec3<f32>(
        x * (fullness + (1.0 - fullness) * across * rise),
        brow + (radii.y - brow) * y + crest,
        radii.z * sin(latitude) * cos(angle) * fullness,
    );
}

// The barbute's bowl, its meridians resampled through the angle table so
// that the face opening has its designed width.
fn bowl_remap(radii: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
    let x = p.x / radii.x;
    let z = p.z / radii.z;
    if (x == 0.0 && z == 0.0) {
        return p;
    }
    let radius = sqrt(x * x + z * z);
    var original = atan2(x, z);
    if (original < 0.0) {
        original = original + TAU;
    }
    let coordinate = original / TAU * AROUND;
    let index = u32(floor(coordinate)) % 48u;
    var next: f32;
    if (index + 1u == 48u) {
        next = design[BOWL_TABLE] + TAU;
    } else {
        next = design[BOWL_TABLE + index + 1u];
    }
    let here = design[BOWL_TABLE + index];
    let angle = here + (next - here) * fract(coordinate);
    return vec3<f32>(radii.x * radius * sin(angle), p.y, radii.z * radius * cos(angle));
}

// The sallet's front arc stretched to its
// opening, the rest of the bowl compressed behind it.
fn front_arc(radii: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
    let x = p.x / radii.x;
    let z = p.z / radii.z;
    if (x == 0.0 && z == 0.0) {
        return p;
    }
    let original = SALLET_ORIGINAL_ARC;
    let opening = design[OPENING_WIDTH];
    let angle = atan2(x, z);
    let absolute = abs(angle);
    var mapped: f32;
    if (absolute <= original) {
        mapped = absolute * opening / original;
    } else {
        mapped = opening + (PI - opening) * (absolute - original) / (PI - original);
    }
    mapped = mapped * select(1.0, -1.0, angle < 0.0);
    let radius = sqrt(x * x + z * z);
    return vec3<f32>(radii.x * radius * sin(mapped), p.y, radii.z * radius * cos(mapped));
}

// The styled crown's formed ridge, from the dome point before any remap.
fn ridge_relief(radii: vec3<f32>, brow: f32, p: vec3<f32>) -> f32 {
    let rise = clamp((p.y - brow) / (radii.y - brow), 0.0, 1.0);
    let medial = max(1.0 - abs(p.x / (radii.x * RIDGE_SPREAD)), 0.0);
    return design[RIDGE] * pow2(medial) * rise;
}

fn dome_vertex(kind: u32, coord: vec4<f32>) -> ShellVertex {
    let radii = dome_radii();
    let brow = brow();
    var p: vec3<f32>;
    var h = 0.0;
    if (kind == KIND_DOME_POLE) {
        p = vec3<f32>(0.0, radii.y, 0.0);
    } else if (kind == KIND_DOME_RING) {
        let latitude = coord.y;
        let angle = coord.z;
        let spread = pow(sin(latitude), design[FULLNESS]);
        p = vec3<f32>(
            radii.x * spread * sin(angle),
            brow + (radii.y - brow) * cos(latitude),
            radii.z * spread * cos(angle),
        );
    } else if (kind == KIND_FAN_RIM) {
        p = vec3<f32>(radii.x * sin(coord.y), brow, radii.z * cos(coord.y));
    } else {
        var latitude: f32;
        var u: f32;
        if (kind == KIND_FAN_POINT) {
            latitude = coord.y;
            u = coord.z;
        } else {
            let width = select(crest_half_width(radii), crest_base_width(radii), coord.z > 0.5);
            latitude = FRAC_PI_2 + coord.y * asin(width / radii.x);
            u = coord.w;
        }
        let v = sin(latitude);
        var fanned = u;
        if (design[FLUTED] > 0.5) {
            fanned = flute_fan(u, v);
            h = flute_relief(u, v);
        }
        p = crown_point(radii, brow, latitude, fanned * PI);
    }
    h = h + ridge_relief(radii, brow, p);
    let map = u32(design[POST_MAP]);
    if (map == MAP_BOWL_TABLE) {
        p = bowl_remap(radii, p);
    } else if (map == MAP_FRONT_ARC) {
        p = front_arc(radii, p);
    }
    return ShellVertex(p, h);
}

fn brim_point(t: f32, angle: f32) -> vec3<f32> {
    let radii = dome_radii();
    let c = cos(angle);
    let end_sweep = pow4(abs(c));
    var reach = design[BACK_REACH];
    var sweep_scale = design[BACK_SWEEP];
    if (c >= 0.0) {
        reach = design[FRONT_REACH];
        sweep_scale = 1.0;
    }
    let extension = design[BRIM_WIDTH] * t * (1.0 + (reach - 1.0) * end_sweep);
    return vec3<f32>(
        (radii.x + extension) * sin(angle),
        brow() + design[BRIM_SWEEP] * sweep_scale * end_sweep * pow2(t) - design[BRIM_DROP] * t,
        (radii.z + extension) * c,
    );
}

// The barbute's cheeks below the eye shelf, with rounded inner corners.
fn barbute_cheek(row: u32, step: f32, column: f32) -> vec3<f32> {
    let radii = dome_radii();
    let half_height = fit.half_extents.y;
    let eye = design[EYE_OPENING];
    let mouth = design[MOUTH_OPENING];
    let shelf = brow() - design[EYE_HEIGHT];
    let corner = design[EYE_HEIGHT] * BARBUTE_CORNER_SHARE * design[OPENING_ROUNDNESS];
    let corner_angle = min(corner / radii.x, (eye - mouth) * 0.35);
    let lower = -half_height * design[CHEEK_DEPTH];
    let start = shelf - corner;
    var y = shelf;
    if (row == ROW_CORNER) {
        y = shelf - corner * step / BARBUTE_CORNER_ROWS;
    } else if (row == ROW_LOWER) {
        y = start + (lower - start) * step / BARBUTE_LOWER_ROWS;
    }
    var lower_blend = 1.0;
    if (corner > BARBUTE_MINIMUM_CORNER_M) {
        lower_blend = clamp((shelf - y) / corner, 0.0, 1.0);
    }
    let inner_round = corner_angle * (1.0 - sqrt(1.0 - pow2(1.0 - lower_blend)));
    let absolute = min(column, TAU - column);
    let shift = inner_round * (1.0 - clamp((absolute - mouth) / (eye - mouth), 0.0, 1.0));
    let angle = column + select(-shift, shift, column < PI);
    var outer_round = 0.0;
    if (row == ROW_SHELF && corner_angle > 1e-6) {
        let t = clamp((absolute - (eye - corner_angle)) / corner_angle, 0.0, 1.0);
        outer_round = corner * (1.0 - sqrt(1.0 - t * t));
    }
    let descent = clamp((shelf - y) / (shelf - lower), 0.0, 1.0);
    let lifted = y + outer_round
        + design[REAR_EDGE_LIFT] * pow2(descent) * pow2((1.0 - cos(angle)) * 0.5);
    let jaw = max(-lifted / half_height, 0.0);
    let basal_return = max((jaw - 0.7) / 0.3, 0.0) * design[NAPE_FLARE] / radii.x;
    let taper = 1.0 - (1.0 - design[CHIN_TAPER]) * pow2(jaw) + basal_return;
    return vec3<f32>(
        radii.x * taper * sin(angle),
        lifted,
        radii.z * (1.0 - 0.12 * pow2(jaw) * max(-cos(angle), 0.0) + basal_return) * cos(angle),
    );
}

// The burgonet's nape plate at a fraction `t` of its depth.
fn nape_point(t: f32, angle: f32) -> vec3<f32> {
    let radii = dome_radii();
    let taper = 1.0 - (1.0 - design[NAPE_TAPER]) * t * t;
    let neck = min(t / NAPE_FLARE_START, 1.0);
    let recession = design[NAPE_RECESSION] * neck * neck * (3.0 - 2.0 * neck);
    let flare = design[NECK_FLARE] * max((t - NAPE_FLARE_START) / (1.0 - NAPE_FLARE_START), 0.0);
    let rear = max(-cos(angle), 0.0);
    let depth = fit.half_extents.y * design[NAPE_DEPTH] * (0.65 + 0.35 * rear * rear);
    let back = cos(angle) * (1.0 - t * t) - power(rear, 0.65) * t * t;
    return vec3<f32>(
        (radii.x * taper + flare * 0.4) * sin(angle),
        brow() - depth * t,
        (radii.z - recession + flare) * back,
    );
}

// The peak's central rise is independent of its downward pitch.
fn peak_point(t: f32, angle: f32) -> vec3<f32> {
    let radii = dome_radii();
    let reach = design[PEAK_LENGTH] * cos(angle) * t;
    return vec3<f32>(
        (radii.x + reach) * sin(angle),
        brow() - design[PEAK_DROP] * cos(angle) * t + peak_rise(angle) * t,
        (radii.z + reach) * cos(angle),
    );
}

// The peak's central rise, fading to nothing at its sides.
fn peak_rise(angle: f32) -> f32 {
    return design[PEAK_RISE]
        * max(1.0 - abs(sin(angle)) / sin(PI / 3.0), 0.0)
        * max(cos(angle), 0.0);
}

// A tangent V-section encloses the curved face rather than cutting into it;
// sharpness blends from the rounded ridge toward the formed flats.
fn formed_ridge(angle: f32, depth: f32, height: f32, sharpness: f32) -> f32 {
    let front = max(cos(angle), 0.0);
    let rounded = height * pow4(front);
    let apex = depth + height;
    var angular = 0.0;
    if (height > 0.0 && front > depth / apex) {
        let slope = sqrt(pow2(apex / depth) - 1.0);
        angular = max(apex - depth * slope * abs(sin(angle)) - depth * front, 0.0);
    }
    return rounded * (1.0 - sharpness) + angular * sharpness;
}

// The buffe's face `u` across and `v` up, from its hem to the sight gap
// under the peak. Broad sections enclose the cheek plates, blending into the
// chin taper below them.
fn buffe_carrier(u: f32, v: f32) -> vec3<f32> {
    let radii = head_radii();
    let brow = brow();
    let half_height = fit.half_extents.y;
    let gauge = design[GAUGE];
    let angle = (2.0 * u - 1.0) * design[BUFFE_SIDE_WRAP];
    let front = max(cos(angle), 0.0);
    let top = brow - (design[BUFFE_SIGHT_GAP] + design[PEAK_DROP]) * front + peak_rise(angle);
    let hem = -half_height - design[BUFFE_NECK_DROP] + design[BUFFE_CHIN_POINT] * abs(sin(angle));
    let chin_width = design[BUFFE_CHIN_WIDTH];
    var width = radii.x * (chin_width + (1.0 - chin_width) * v) + gauge * 3.0;
    let throat = design[BUFFE_THROAT_DEPTH];
    let depth = radii.z * (throat + (1.0 - throat) * v);
    let sharpness = design[BUFFE_RIDGE_SHARPNESS];
    let ridge_height = design[BUFFE_RIDGE] * sin(PI * v);
    var ridge: f32;
    if (design[BUFFE_COURSES] > 0.0) {
        ridge = formed_ridge(angle, depth, ridge_height, sharpness);
    } else {
        var angular = 0.0;
        if (front > 0.0) {
            angular = 1.0 - abs(sin(angle));
        }
        ridge = ridge_height * (pow4(front) * (1.0 - sharpness) + angular * sharpness);
    }
    let y = hem + (top - hem) * v;
    let z = depth * cos(angle) + design[BUFFE_PROJECTION] * v * pow4(front) + ridge;
    let lower = clamp((brow - y) / half_height, 0.0, 1.0);
    let cheek_bottom = brow - half_height * (0.40 + 0.46 * design[CHEEK_DEPTH]) - design[CHIN_TAB];
    var blend = clamp((y - cheek_bottom + 0.040) / 0.040, 0.0, 1.0);
    blend = blend * blend * (3.0 - 2.0 * blend);
    let cheek_width = radii.x * (1.0 - (1.0 - design[CHEEK_TAPER]) * lower) + gauge * 5.0;
    width = width + max(cheek_width - width, 0.0) * blend;
    return vec3<f32>(width * sin(angle), y, z);
}

// Where course `course` spans the face at `u`: its lower edge dropped to a
// chevron and lapped over the course below, its upper edge dropped to the
// next course's chevron.
fn buffe_course_extents(u: f32, course: f32) -> vec2<f32> {
    let angle = (2.0 * u - 1.0) * design[BUFFE_SIDE_WRAP];
    let height = buffe_carrier(u, 1.0).y - buffe_carrier(u, 0.0).y;
    var chevron = 0.0;
    if (cos(angle) > 0.0) {
        chevron = 1.0 - abs(sin(angle));
    }
    let drop = design[BUFFE_BOUNDARY_DROP] * chevron / height;
    var bounds = array<f32, 4>(0.0, design[BUFFE_LOWER], design[BUFFE_UPPER], 1.0);
    let index = u32(course);
    var high = 1.0;
    if (index + 1u < u32(design[BUFFE_COURSES])) {
        high = bounds[index + 1u] - drop;
    }
    var low = 0.0;
    if (index > 0u) {
        low = bounds[index] - drop - design[BUFFE_OVERLAP] / height;
    }
    return vec2<f32>(low, high);
}

fn buffe_point(u: f32, v: f32, course: f32) -> vec3<f32> {
    if (design[BUFFE_COURSES] == 0.0) {
        return buffe_carrier(u, v);
    }
    let extents = buffe_course_extents(u, course);
    var p = buffe_carrier(u, extents.x + (extents.y - extents.x) * v);
    // Each lower edge is formed over the course beneath it; the fitted sight
    // and exposed top edges stay where they are.
    if (course > 0.0) {
        let angle = (2.0 * u - 1.0) * design[BUFFE_SIDE_WRAP];
        let lap = (2.0 * design[GAUGE] + design[BUFFE_LAP_CLEARANCE]) * (1.0 - v);
        p.x = p.x + lap * sin(angle);
        p.z = p.z + lap * cos(angle);
    }
    return p;
}

fn sallet_skirt(t: f32, angle: f32) -> vec3<f32> {
    let radii = dome_radii();
    let rear = max(-cos(angle), 0.0);
    let extension = design[TAIL_LENGTH] * pow3(t);
    let tail = power(rear, 2.0 / design[TAIL_WIDTH]);
    return vec3<f32>(
        (radii.x + extension * tail * 0.12) * sin(angle) * (1.0 - 0.35 * tail * t * t),
        brow() - fit.half_extents.y * design[CHEEK_DEPTH] * t - design[TAIL_DROP] * tail * t
            + design[REAR_EDGE_LIFT] * pow2((1.0 - cos(angle)) * 0.5) * pow2(t),
        (radii.z + extension * tail) * cos(angle),
    );
}

// The visor follows the bowl a spacing outside it, its narrow pivot arms
// rising at the sides.
fn visor_point(v: f32, angle: f32) -> vec3<f32> {
    let radii = head_radii();
    let brow = brow();
    let top = brow - design[SIGHT_GAP];
    let rise = design[PIVOT_RISE];
    let side = abs(angle) / FRAC_PI_2;
    let arm = pow4(side) * side;
    let upper = top + rise * arm;
    let tip = max((side - 0.88) / 0.12, 0.0);
    let panel = design[VISOR_HEIGHT] * (1.0 - (1.0 - design[SIDE_PANEL]) * arm) + rise * arm;
    let height = panel + (VISOR_ARM_WIDTH_M - panel) * tip * tip;
    let projection = design[VISOR_PROJECTION] * pow2(1.0 - v) * pow2(cos(angle));
    let y = upper - height * v;
    let latitude = clamp((y - brow) / (radii.y - brow), 0.0, 0.99);
    let bowl = pow(1.0 - latitude * latitude, design[FULLNESS] * 0.5);
    let spacing = design[VISOR_SPACING];
    return vec3<f32>(
        (radii.x * bowl + spacing) * sin(angle),
        y,
        ((radii.z + design[Z_EXTENSION]) * bowl + spacing) * cos(angle) + projection,
    );
}

// A burgonet cheek plate, fanned from its root toward its rounded edge.
fn cheek_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    let radii = dome_radii();
    let brow = brow();
    let half_height = fit.half_extents.y;
    let width = radii.z * 0.38 * design[CHEEK_WIDTH];
    let height = half_height * 0.46 * design[CHEEK_DEPTH];
    let center = vec2<f32>(brow - half_height * 0.40, radii.z * 0.10);
    let root = vec2<f32>(center.x - height * 0.65, center.y + width * 0.55);
    var q = root;
    if (index != 0u) {
        let v = coord.x;
        let angle = coord.y;
        let tab = coord.z;
        let edge = vec2<f32>(
            center.x + height * cos(angle) - tab * 0.8,
            center.y - width * sin(angle) + tab * 0.6,
        );
        q = vec2<f32>(root.x + (edge.x - root.x) * v, root.y + (edge.y - root.y) * v);
    }
    let depth = clamp(q.y / radii.z, -0.98, 0.98);
    let lowered = clamp((brow - q.x) / half_height, 0.0, 1.0);
    let p = vec3<f32>(
        radii.x * (1.0 - (1.0 - design[CHEEK_TAPER]) * lowered) * sqrt(1.0 - depth * depth),
        q.x,
        q.y,
    );
    return ShellVertex(p, coord.w);
}

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    if (params.value0 == CHEEK_SHELL) {
        return cheek_vertex(index, coord);
    }
    let kind = u32(coord.x);
    if (kind == KIND_BRIM) {
        return ShellVertex(brim_point(coord.y, coord.z), 0.0);
    }
    if (kind == KIND_BARBUTE_CHEEK) {
        return ShellVertex(barbute_cheek(u32(coord.y), coord.z, coord.w), 0.0);
    }
    if (kind == KIND_NAPE) {
        let v = coord.z;
        let angle = coord.w;
        let offset = design[GAUGE] * 2.0 * v * v;
        let p = nape_point(coord.y, angle);
        return ShellVertex(vec3<f32>(p.x + offset * sin(angle), p.y, p.z + offset * cos(angle)), 0.0);
    }
    if (kind == KIND_PEAK) {
        return ShellVertex(peak_point(coord.y, coord.z), 0.0);
    }
    if (kind == KIND_GUARD) {
        return ShellVertex(nape_point(coord.y, coord.z), 0.0);
    }
    if (kind == KIND_SALLET_SKIRT) {
        return ShellVertex(sallet_skirt(coord.y, coord.z), 0.0);
    }
    if (kind == KIND_VISOR) {
        return ShellVertex(visor_point(coord.y, coord.z), 0.0);
    }
    if (kind == KIND_BUFFE) {
        return ShellVertex(buffe_point(coord.y, coord.z, coord.w), 0.0);
    }
    return dome_vertex(kind, coord);
}
"#;
