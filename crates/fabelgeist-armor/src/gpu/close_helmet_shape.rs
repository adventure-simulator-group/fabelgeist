//! The close helmet's carrier on the device, evaluated per vertex from the
//! wearer's measured sections.
//!
//! Every shell of the helmet shares this shape. A vertex's coordinate names
//! its kind in its fourth float; `params.value0` names its shell.
//!
//! The bevor, visor and nape lames sit whole gauges outside the carrier along
//! finite-difference normals, which magnify every rounding difference forty
//! times; the pierced visor's thin return walls then turn those differences
//! into different normals. So the carrier is evaluated in a fixed order of
//! operations, independent of the device's fused operations: design angles'
//! sines and cosines are computed with the design on the host, and every
//! product, quotient and root is exactly rounded (see
//! [`fabelgeist_compute::host_float`]).

/// Design floats, in [`super::close_helmet`]'s order.
pub(crate) const DESIGN: &str = r#"
const WALL: u32 = 0u;
const GAP: u32 = 1u;
const CROWN_HEIGHT: u32 = 2u;
const BACK_EDGE_LIFT: u32 = 3u;
const NECK_LENGTH: u32 = 4u;
const JAW_WIDTH: u32 = 5u;
const NECK_WIDTH: u32 = 6u;
const THROAT_FLARE: u32 = 7u;
const BACK_FLARE: u32 = 8u;
const VISOR_PROJECTION: u32 = 9u;
const CHIN_PROJECTION: u32 = 10u;
const NAPE_LENGTH: u32 = 11u;
const NAPE_FLARE: u32 = 12u;
const VISOR_RIDGE: u32 = 13u;
const RIDGE_SHARPNESS: u32 = 14u;
const SIGHT_LEDGE: u32 = 15u;
const CROWN_FULLNESS: u32 = 16u;
const CROWN_RIDGE: u32 = 17u;
const COMB: u32 = 18u;
const CROWN_FLUTED: u32 = 19u;
const SIDE_WRAP_COSINE: u32 = 20u;
// Always zero; the device cannot see that it is.
const ZERO: u32 = 21u;
const CROWN_FLUTE: u32 = 22u;

// Floats per vertex in a shell's turn table, and where its extra value sits.
const TURN_WORDS: u32 = 8u;
const TURN_EXTRA: u32 = 6u;

// The wearer's sections follow the frame in the fit buffer.
const PROFILE: u32 = 16u;
const SKULL_HALF_WIDTH: u32 = 0u;
const TEMPLE_HALF_WIDTH: u32 = 1u;
const SKULL_FRONT: u32 = 2u;
const SKULL_BACK: u32 = 3u;
const JAW_HALF_WIDTH: u32 = 4u;
const JAW_FRONT: u32 = 5u;
const SUBMENTAL_FRONT: u32 = 6u;
const NECK_HALF_WIDTH: u32 = 7u;
const THROAT_FRONT: u32 = 8u;
const NAPE_BACK: u32 = 9u;
const NAPE_WAIST: u32 = 10u;

// Shells, in `params.value0`; `params.value1` is where the shell's turn
// table starts in the design floats.
const SHELL_SKULL: f32 = 0.0;
const SHELL_NAPE: f32 = 1.0;
const SHELL_BEVOR: f32 = 2.0;
const SHELL_VISOR: f32 = 3.0;

// Skull vertex kinds past the dome's.
const KIND_JAW_ROW: u32 = 4u;
"#;

pub(crate) const SHAPE: &str = r#"
const JAW_ROWS: f32 = 12.0;
const NECK_ROWS: f32 = 6.0;
const PLATE_GAP_M: f32 = 0.002;
const VISOR_BROW_OVERLAP_M: f32 = 0.030;
const NECK_HEM_HEAD_RATIO: f32 = 1.20;
const CHIN_HEAD_RATIO: f32 = 1.03;
const EAR_LOBE_TAPER_START: f32 = 0.40;
const LOWER_FACE_PROJECTION_FRACTION: f32 = 0.25;
const VISOR_SEATING_TRANSITION: f32 = 0.25;
const VISOR_LOWER_EDGE_HEAD_RATIO: f32 = 0.86;
const BEVOR_MOUTH_HEAD_RATIO: f32 = 0.76;
const FACE_PROJECTION_HEAD_RATIO: f32 = 0.35;
const SIGHT_BROW_DROP_HEAD_RATIO: f32 = 0.07;
const BROW_PEAK_M: f32 = 0.025;
const BEVOR_PIVOT_BROW_DROP_M: f32 = 0.008;
const SUBMENTAL_NECK_FRACTION: f32 = 0.35;
const BROW_HEIGHT: f32 = 0.09;
// The sight's centre down the authored visor: 30 mm of its 100.
const SIGHT_T: f32 = 0.3;
const SIGHT_LEDGE_RISE: f32 = 0.10;
const SIGHT_LEDGE_FALL: f32 = 0.16;
const NORMAL_SAMPLE_HEIGHT_M: f32 = 0.0001;
const NAPE_ROOT_RISE_M: f32 = 0.030;
const NAPE_CORNER_SWEEP: f32 = 0.20;
const NAPE_SAMPLE: f32 = 0.001;
const NAPE_SWEEP_ACROSS: f32 = 0.9;
const HINGE_HEIGHT_FRACTION: f32 = 0.20;

fn section_value(k: u32) -> f32 {
    return frames[PROFILE + k];
}

fn half_height() -> f32 {
    return fit.half_extents.y;
}

fn brow() -> f32 {
    return host_mul(half_height(), BROW_HEIGHT);
}

fn crown() -> f32 {
    return host_add(host_mul(half_height(), design[CROWN_HEIGHT]), design[GAP]);
}

fn skull_center() -> f32 {
    return host_mul(host_add(section_value(SKULL_FRONT), section_value(SKULL_BACK)), 0.5);
}

fn skull_depth() -> f32 {
    return host_mul(host_sub(section_value(SKULL_FRONT), section_value(SKULL_BACK)), 0.5);
}

fn exact_square(x: f32) -> f32 {
    return host_mul(x, x);
}

fn exact_pow4(x: f32) -> f32 {
    return exact_square(exact_square(x));
}

// A design angle's sine and cosine, computed with the design on the host.
struct Turn {
    s: f32,
    c: f32,
};

// Each vertex's turns in its shell's table: at its angle, then a sample
// either side, then one more design value.
fn table(index: u32, k: u32) -> f32 {
    return design[u32(params.value1) + index * TURN_WORDS + k];
}

fn turn_at(index: u32, sample: u32) -> Turn {
    return Turn(table(index, sample * 2u), table(index, sample * 2u + 1u));
}

fn front_weight(turn: Turn) -> f32 {
    return exact_square(max(turn.c, 0.0));
}

fn edge_lift(turn: Turn) -> f32 {
    return host_mul(design[BACK_EDGE_LIFT], host_sub(1.0, front_weight(turn)));
}

fn jaw_height(turn: Turn) -> f32 {
    return host_add(host_mul(-half_height(), CHIN_HEAD_RATIO), edge_lift(turn));
}

fn hem_height(turn: Turn) -> f32 {
    let hem = host_sub(host_mul(-half_height(), NECK_HEM_HEAD_RATIO), design[NECK_LENGTH]);
    return host_add(hem, edge_lift(turn));
}

fn row_height(top: f32, turn: Turn, row: f32) -> f32 {
    let jaw = jaw_height(turn);
    if (row <= JAW_ROWS) {
        return host_add(top, host_div(host_mul(host_sub(jaw, top), row), JAW_ROWS));
    }
    let span = host_sub(hem_height(turn), jaw);
    return host_add(jaw, host_div(host_mul(span, host_sub(row, JAW_ROWS)), NECK_ROWS));
}

// The measured head section at a turn: its width and its front and back
// depths, blended from the temples to the jaw and down the neck, as (x, z).
fn section(turn: Turn, jaw_blend: f32, neck_fraction: f32, back_blend: f32) -> vec2<f32> {
    let neck_blend = host_mul(
        exact_square(neck_fraction),
        host_sub(3.0, host_mul(2.0, neck_fraction)),
    );
    let upper_width = section_value(TEMPLE_HALF_WIDTH);
    let jaw_width = max(section_value(JAW_HALF_WIDTH), host_mul(upper_width, design[JAW_WIDTH]));
    let neck_width = max(section_value(NECK_HALF_WIDTH), host_mul(upper_width, design[NECK_WIDTH]));
    let width = host_lerp(host_lerp(upper_width, jaw_width, jaw_blend), neck_width, neck_blend);
    var front: f32;
    if (neck_fraction <= SUBMENTAL_NECK_FRACTION) {
        front = host_lerp(
            host_lerp(section_value(SKULL_FRONT), section_value(JAW_FRONT), jaw_blend),
            section_value(SUBMENTAL_FRONT),
            host_div(neck_fraction, SUBMENTAL_NECK_FRACTION),
        );
    } else {
        front = host_lerp(
            section_value(SUBMENTAL_FRONT),
            section_value(THROAT_FRONT),
            host_div(
                host_sub(neck_fraction, SUBMENTAL_NECK_FRACTION),
                host_sub(1.0, SUBMENTAL_NECK_FRACTION),
            ),
        );
    }
    let back = host_lerp(
        host_lerp(section_value(SKULL_BACK), section_value(NAPE_WAIST), back_blend),
        section_value(NAPE_BACK),
        neck_blend,
    );
    let center = host_mul(host_add(front, back), 0.5);
    return vec2<f32>(
        host_mul(width, turn.s),
        host_add(center, host_mul(host_mul(host_sub(front, back), 0.5), turn.c)),
    );
}

// The carrier above the brow: the skull's ellipsoid.
fn skull_point(turn: Turn, y: f32, brow: f32) -> vec3<f32> {
    let height = clamp(host_div(host_sub(y, brow), host_sub(crown(), brow)), 0.0, 1.0);
    let radius = host_sqrt(host_sub(1.0, exact_square(height)));
    let temple = section_value(TEMPLE_HALF_WIDTH);
    let widening = host_sub(section_value(SKULL_HALF_WIDTH), temple);
    let width = host_add(temple, host_mul(widening, host_smoothstep(height)));
    return vec3<f32>(
        host_mul(host_mul(width, radius), turn.s),
        y,
        host_add(skull_center(), host_mul(host_mul(skull_depth(), radius), turn.c)),
    );
}

// A carrier point before any plate offset: the skull above the brow, the
// sections below it, with the neck's flare, the face's projection and the
// chin.
fn base_point(turn: Turn, y: f32) -> vec3<f32> {
    let brow = brow();
    if (y > brow) {
        return skull_point(turn, y, brow);
    }
    let jaw = jaw_height(turn);
    let t = host_div(host_sub(brow, y), host_sub(brow, jaw));
    let jaw_blend = host_smoothstep(host_div(
        host_sub(t, EAR_LOBE_TAPER_START),
        host_sub(1.0, EAR_LOBE_TAPER_START),
    ));
    let neck_fraction = clamp(
        host_div(host_sub(jaw, y), host_sub(jaw, hem_height(turn))),
        0.0,
        1.0,
    );
    let neck_blend = host_smoothstep(neck_fraction);
    let xz = section(turn, jaw_blend, neck_fraction, host_smoothstep(t));
    let front = front_weight(turn);
    let flare = host_add(
        host_mul(design[THROAT_FLARE], front),
        host_mul(design[BACK_FLARE], host_sub(1.0, front)),
    );
    let lip = host_mul(host_mul(flare, exact_pow4(neck_blend)), 0.5);
    let mouth = host_mul(-half_height(), FACE_PROJECTION_HEAD_RATIO);
    var face_blend: f32;
    if (y >= mouth) {
        face_blend = host_smoothstep(host_div(host_sub(brow, y), host_sub(brow, mouth)));
    } else {
        face_blend = clamp(host_div(host_sub(jaw, y), host_sub(jaw, mouth)), 0.0, 1.0);
    }
    let face_projection = host_mul(
        host_mul(design[VISOR_PROJECTION], LOWER_FACE_PROJECTION_FRACTION),
        face_blend,
    );
    let chin = host_mul(
        host_mul(host_mul(design[CHIN_PROJECTION], front), jaw_blend),
        host_sub(1.0, neck_blend),
    );
    let z = host_add(
        host_add(host_add(xz.y, host_mul(lip, turn.c)), host_mul(face_projection, front)),
        chin,
    );
    return vec3<f32>(host_add(xz.x, host_mul(lip, turn.s)), y, z);
}

// A point `offset` along the unit of a finite-difference normal.
fn offset_along(p: vec3<f32>, normal: vec3<f32>, offset: f32) -> vec3<f32> {
    let length = host_sqrt(host_dot(normal, normal));
    return host_add3(p, vec3<f32>(
        host_mul(host_div(normal.x, length), offset),
        host_mul(host_div(normal.y, length), offset),
        host_mul(host_div(normal.z, length), offset),
    ));
}

fn plate_offset(layer: f32) -> f32 {
    return host_mul(host_add(design[WALL], PLATE_GAP_M), layer);
}

// The carrier at a vertex's own turns: overlapping plates sit whole
// gauges outside the carrier, along its finite-difference normal.
fn point(index: u32, y: f32, layer: f32) -> vec3<f32> {
    let turn = turn_at(index, 0u);
    let p = base_point(turn, y);
    if (layer == 0.0) {
        return p;
    }
    let across = host_sub3(base_point(turn_at(index, 1u), y), base_point(turn_at(index, 2u), y));
    let up = host_sub3(
        base_point(turn, host_add(y, NORMAL_SAMPLE_HEIGHT_M)),
        base_point(turn, host_sub(y, NORMAL_SAMPLE_HEIGHT_M)),
    );
    return offset_along(p, host_cross(across, up), plate_offset(layer));
}

// The nape's root at `u` across it, whose angle's turn is given.
fn nape_base(turn: Turn, u: f32, t: f32) -> vec3<f32> {
    let rise = host_mul(NAPE_ROOT_RISE_M, host_sub(1.0, exact_square(u)));
    var p = base_point(turn, host_add(hem_height(turn), rise));
    let sweep = host_mul(design[NAPE_FLARE], t);
    let corner = host_add(1.0, host_mul(host_mul(NAPE_CORNER_SWEEP, u), u));
    p.x = host_sub(p.x, host_mul(host_mul(sweep, u), NAPE_SWEEP_ACROSS));
    p.y = host_sub(p.y, host_mul(design[NAPE_LENGTH], t));
    p.z = host_sub(p.z, host_mul(sweep, corner));
    return p;
}

fn nape_point(index: u32, u: f32, t: f32, layer: f32) -> vec3<f32> {
    let turn = turn_at(index, 0u);
    let across = host_sub3(
        nape_base(turn_at(index, 1u), host_add(u, NAPE_SAMPLE), t),
        nape_base(turn_at(index, 2u), host_sub(u, NAPE_SAMPLE), t),
    );
    let up = host_sub3(
        nape_base(turn, u, host_sub(t, NAPE_SAMPLE)),
        nape_base(turn, u, host_add(t, NAPE_SAMPLE)),
    );
    return offset_along(nape_base(turn, u, t), host_cross(across, up), plate_offset(layer));
}

fn bevor_point(index: u32, row: f32) -> vec3<f32> {
    let turn = turn_at(index, 0u);
    let front = max(turn.c, 0.0);
    let brow = brow();
    let pivot = host_sub(brow, BEVOR_PIVOT_BROW_DROP_M);
    let mouth = host_sub(
        host_add(host_mul(half_height(), BEVOR_MOUTH_HEAD_RATIO), brow),
        BEVOR_PIVOT_BROW_DROP_M,
    );
    let top = host_sub(pivot, host_mul(mouth, exact_square(front)));
    return point(index, row_height(top, turn, row), 1.0);
}

fn visor_height() -> f32 {
    return host_add(
        host_add(brow(), VISOR_BROW_OVERLAP_M),
        host_mul(half_height(), VISOR_LOWER_EDGE_HEAD_RATIO),
    );
}

fn visor_y(turn: Turn, t: f32) -> f32 {
    let brow = brow();
    let eye = host_sub(brow, host_mul(half_height(), SIGHT_BROW_DROP_HEAD_RATIO));
    if (t <= SIGHT_T) {
        let top = host_add(
            host_add(brow, VISOR_BROW_OVERLAP_M),
            host_mul(BROW_PEAK_M, front_weight(turn)),
        );
        return host_add(top, host_div(host_mul(host_sub(eye, top), t), SIGHT_T));
    }
    let bottom = host_mul(-half_height(), VISOR_LOWER_EDGE_HEAD_RATIO);
    let fall = host_mul(host_sub(bottom, eye), host_sub(t, SIGHT_T));
    return host_add(eye, host_div(fall, host_sub(1.0, SIGHT_T)));
}

// The pierced lifting visor two gauges out, its ridge and sight ledge
// projecting forward. The table's extra value is the ridge's rounded ramp.
fn visor_point(index: u32, t: f32) -> vec3<f32> {
    let ridge = design[VISOR_RIDGE];
    var ramp: f32;
    if (t <= ridge) {
        ramp = host_div(t, ridge);
    } else {
        ramp = host_div(host_sub(1.0, t), host_sub(1.0, ridge));
    }
    let rounded = table(index, TURN_EXTRA);
    let profile = host_add(rounded, host_mul(host_sub(ramp, rounded), design[RIDGE_SHARPNESS]));
    let projection = host_mul(design[VISOR_PROJECTION], profile);
    let turn = turn_at(index, 0u);
    var p = point(index, visor_y(turn, t), 2.0);
    let front = front_weight(turn);
    let seating = host_smoothstep(host_div(host_sub(1.0, t), VISOR_SEATING_TRANSITION));
    p.z = host_add(p.z, host_mul(host_mul(projection, seating), front));
    let below_sight = host_sub(t, SIGHT_T);
    let ledge = host_mul(
        host_smoothstep(host_div(below_sight, SIGHT_LEDGE_RISE)),
        host_sub(
            1.0,
            host_smoothstep(host_div(host_sub(below_sight, SIGHT_LEDGE_RISE), SIGHT_LEDGE_FALL)),
        ),
    );
    p.z = host_add(p.z, host_mul(host_mul(design[SIGHT_LEDGE], ledge), front));
    return p;
}

// The styled bowl, widened toward the crown and centred on the skull.
fn skull_dome(coord: vec4<f32>) -> ShellVertex {
    let temple = section_value(TEMPLE_HALF_WIDTH);
    let crown = crown();
    let brow = brow();
    let d = Dome(
        vec3<f32>(temple, crown, skull_depth()),
        brow,
        design[CROWN_FULLNESS],
        design[CROWN_RIDGE],
        design[COMB],
        design[CROWN_FLUTED] != 0.0,
        CROWN_FLUTE,
    );
    var v = dome_vertex(d, coord);
    let crown_blend = host_smoothstep(host_div(host_sub(v.point.y, brow), host_sub(crown, brow)));
    let widening = host_sub(host_div(section_value(SKULL_HALF_WIDTH), temple), 1.0);
    v.point.x = host_mul(v.point.x, host_add(1.0, host_mul(widening, crown_blend)));
    v.point.z = host_add(v.point.z, skull_center());
    return v;
}

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    if (params.value0 == SHELL_NAPE) {
        return ShellVertex(nape_point(index, coord.x, coord.y, coord.z), 0.0);
    }
    if (params.value0 == SHELL_BEVOR) {
        return ShellVertex(bevor_point(index, coord.y), 0.0);
    }
    if (params.value0 == SHELL_VISOR) {
        return ShellVertex(visor_point(index, coord.y), coord.z);
    }
    if (u32(coord.w) == KIND_JAW_ROW) {
        let turn = turn_at(index, 0u);
        return ShellVertex(base_point(turn, row_height(brow(), turn, coord.y)), 0.0);
    }
    return skull_dome(coord);
}

fn shell_pass(index: u32, coord: vec4<f32>) -> u32 {
    return 0u;
}

// The visor thickens radially about the skull's vertical axis.
fn shell_origin() -> vec3<f32> {
    return vec3<f32>(0.0, 0.0, skull_center());
}

// Bevor and visor pivot together at the temples.
fn shell_hinge() -> array<vec3<f32>, 2> {
    return array<vec3<f32>, 2>(
        vec3<f32>(
            0.0,
            host_sub(
                host_add(brow(), VISOR_BROW_OVERLAP_M),
                host_mul(visor_height(), HINGE_HEIGHT_FRACTION),
            ),
            host_add(skull_center(), host_mul(skull_depth(), design[SIDE_WRAP_COSINE])),
        ),
        vec3<f32>(1.0, 0.0, 0.0),
    );
}
"#;
