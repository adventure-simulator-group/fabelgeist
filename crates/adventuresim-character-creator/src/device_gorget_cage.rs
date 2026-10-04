//! The gorget's collar cage as WGSL: the collar and bib surfaces and the
//! bib's seating offset over the measurements a gorget fit holds.
//!
//! The same surfaces serve the bib measurement, which reads the fit as
//! `fit`, and the plate shells, which read it as `frames`; each defines
//! `cage_word(i)` to say which.

/// Where a gorget fit keeps its cage, after the part frame.
pub(crate) const CENTER: u32 = 16;
pub(crate) const COLLAR_RADIUS: u32 = 18;
pub(crate) const BASE_RADIUS: u32 = 20;
pub(crate) const OUTER_WIDTH: u32 = 21;
/// The collar's top plane, then its base plane: height and pitch each.
pub(crate) const PLANES: u32 = 23;
pub(crate) const HEM: u32 = 27;
pub(crate) const FRONT: u32 = 30;
pub(crate) const BACK: u32 = 33;
pub(crate) const CROWN: u32 = 36;
pub(crate) const SIDE_RISE: u32 = 37;
pub(crate) const HEM_FLATNESS: u32 = 38;
pub(crate) const REAR_HEM_FLATNESS: u32 = 39;
pub(crate) const REAR_SWEEP: u32 = 40;
pub(crate) const HEIGHT: u32 = 41;
/// Broad horizontal sections: the shoulders, then three down the front and
/// three down the back, each a low and a high bound.
pub(crate) const BANDS: u32 = 42;
pub(crate) const BAND_COUNT: u32 = 7;
/// The bib's measured seating offsets, then the same smoothed.
pub(crate) const BIB_ROWS: u32 = 17;
pub(crate) const BIB_COLUMNS: u32 = 129;
pub(crate) const BIB_RAW: u32 = BANDS + BAND_COUNT * 6;
pub(crate) const BIB: u32 = BIB_RAW + BIB_ROWS * BIB_COLUMNS;
pub(crate) const COLLAR_RAW: u32 = BIB + BIB_ROWS * BIB_COLUMNS;
pub(crate) const COLLAR: u32 = COLLAR_RAW + BIB_ROWS * BIB_COLUMNS;
pub(crate) const GORGET_FIT_WORDS: u32 = COLLAR + BIB_ROWS * BIB_COLUMNS;

/// The layout constants, for WGSL.
pub(crate) fn layout() -> String {
    [
        ("CENTER", CENTER),
        ("COLLAR_RADIUS", COLLAR_RADIUS),
        ("BASE_RADIUS", BASE_RADIUS),
        ("OUTER_WIDTH", OUTER_WIDTH),
        ("PLANES", PLANES),
        ("HEM", HEM),
        ("FRONT", FRONT),
        ("BACK", BACK),
        ("CROWN", CROWN),
        ("SIDE_RISE", SIDE_RISE),
        ("HEM_FLATNESS", HEM_FLATNESS),
        ("REAR_HEM_FLATNESS", REAR_HEM_FLATNESS),
        ("REAR_SWEEP", REAR_SWEEP),
        ("HEIGHT", HEIGHT),
        ("BANDS", BANDS),
        ("BAND_COUNT", BAND_COUNT),
        ("BIB_ROWS", BIB_ROWS),
        ("BIB_COLUMNS", BIB_COLUMNS),
        ("BIB_RAW", BIB_RAW),
        ("BIB", BIB),
        ("COLLAR_RAW", COLLAR_RAW),
        ("COLLAR", COLLAR),
    ]
    .iter()
    .map(|(name, value)| format!("const {name}: u32 = {value}u;\n"))
    .collect()
}

/// The collar and bib surfaces; needs `cage_word`, `PI`, `TAU` and `pow2`.
pub(crate) const CAGE: &str = r#"
// `f32::rem_euclid(x, TAU)` for |x| below two turns.
fn cage_rem_tau(x: f32) -> f32 {
    var r = x - TAU * trunc(x / TAU);
    if (r < 0.0) {
        r = r + TAU;
    }
    return r;
}

fn cage_vector(at: u32) -> vec3<f32> {
    return vec3<f32>(cage_word(at), cage_word(at + 1u), cage_word(at + 2u));
}

// `powf` for the non-negative bases of the polar chart.
fn polar_power(x: f32, power: f32) -> f32 {
    if (x == 0.0) {
        return 0.0;
    }
    return pow(x, power);
}

// A true polar chart keeps anatomical angular boundaries fixed while its
// cross-section changes from elliptical neck to rounded shoulder bib.
fn polar_radius(width: f32, depth: f32, power: f32, angle: f32) -> f32 {
    let sum = polar_power(abs(sin(angle) / width), power) + polar_power(abs(cos(angle) / depth), power);
    return pow(sum, -1.0 / power);
}

// Rear-facing angles remapped about the back by scaling their lateral
// component; front-facing angles are unchanged.
fn rear_angle(angle: f32, scale: f32) -> f32 {
    if (cos(angle) >= 0.0) {
        return angle;
    }
    let rear = cage_rem_tau(angle) - PI;
    return PI + atan2(scale * sin(rear), max(cos(rear), 0.0));
}

fn surface_angle(control: f32) -> f32 {
    return rear_angle(control, 1.0 - 0.9 * cage_word(REAR_SWEEP));
}

fn control_angle(physical: f32) -> f32 {
    return rear_angle(physical, 1.0 / (1.0 - 0.9 * cage_word(REAR_SWEEP)));
}

// The posterior neck flattens across the trapezius; the throat stays rounded.
fn collar_power(angle: f32) -> f32 {
    return select(3.0, 2.0, cos(angle) >= 0.0);
}

fn plane_at(plane: u32, z: f32) -> f32 {
    return cage_word(PLANES + plane * 2u) - cage_word(PLANES + plane * 2u + 1u) * z;
}

// The front or back depth down the bib: a quadratic through its three
// measured depths.
fn depth_at(front: bool, t: f32) -> f32 {
    let at = select(BACK, FRONT, front);
    return cage_word(at) * (1.0 - t) * (1.0 - 2.0 * t)
        + cage_word(at + 1u) * 4.0 * t * (1.0 - t)
        + cage_word(at + 2u) * t * (2.0 * t - 1.0);
}

fn base_height(angle: f32, z: f32) -> f32 {
    return plane_at(1u, z) + cage_word(SIDE_RISE) * pow2(sin(angle));
}

fn hem_height(angle: f32) -> f32 {
    let cosine = cos(angle);
    let front = cosine >= 0.0;
    let end = select(cage_word(HEM + 2u), cage_word(HEM), front);
    let flatness = select(cage_word(REAR_HEM_FLATNESS), cage_word(HEM_FLATNESS), front);
    let middle = cage_word(HEM + 1u);
    return middle + (end - middle)
        * (cosine * cosine / (cosine * cosine + (1.0 - 0.975 * flatness) * pow2(sin(angle))));
}

// A cubic Hermite with slopes limited to keep each interval
// monotone.
fn bib_cubic(p: vec4<f32>, t: f32) -> f32 {
    let delta = p[2] - p[1];
    if (abs(delta) < 1e-8) {
        return p[1];
    }
    var a = max((p[2] - p[0]) * 0.5 / delta, 0.0);
    var b = max((p[3] - p[1]) * 0.5 / delta, 0.0);
    let magnitude = sqrt(a * a + b * b);
    if (magnitude > 3.0) {
        a = a * (3.0 / magnitude);
        b = b * (3.0 / magnitude);
    }
    let t2 = t * t;
    let t3 = t2 * t;
    return (2.0 * t3 - 3.0 * t2 + 1.0) * p[1]
        + (t3 - 2.0 * t2 + t) * a * delta
        + (-2.0 * t3 + 3.0 * t2) * p[2]
        + (t3 - t2) * b * delta;
}

// Lift over the trapezius at the sides, turning into anterior and
// posterior depth.
fn bib_direction(angle: f32) -> vec3<f32> {
    let up = pow2(sin(angle));
    let back = cos(angle);
    let length = sqrt(up * up + back * back);
    return vec3<f32>(0.0, up / length, back / length);
}

fn bib_value(row: i32, column: i32) -> f32 {
    let r = u32(clamp(row, 0, i32(BIB_ROWS) - 1));
    let c = u32(clamp(column, 0, i32(BIB_COLUMNS) - 1));
    return cage_word(BIB + r * BIB_COLUMNS + c);
}

// The bib's smoothed seating offset along its direction, bicubically
// interpolated over the offset grid.
fn bib_offset(t: f32, angle: f32) -> vec3<f32> {
    let u = cage_rem_tau(angle) / TAU;
    let row = clamp(t, 0.0, 1.0) * f32(BIB_ROWS - 1u);
    let column = u * f32(BIB_COLUMNS - 1u);
    let row_base = i32(floor(row));
    let column_base = i32(floor(column));
    var values: vec4<f32>;
    for (var i = 0; i < 4; i = i + 1) {
        values[i] = bib_cubic(
            vec4<f32>(
                bib_value(row_base + i - 1, column_base - 1),
                bib_value(row_base + i - 1, column_base),
                bib_value(row_base + i - 1, column_base + 1),
                bib_value(row_base + i - 1, column_base + 2),
            ),
            column - trunc(column),
        );
    }
    return bib_direction(angle) * bib_cubic(values, row - trunc(row));
}

// Bilinear measured collar support. Each cell stores an outward envelope;
// convex interpolation cannot introduce the undershoot of a cubic spline.
fn collar_offset(t: f32, angle: f32) -> vec3<f32> {
    let row = clamp(t, 0.0, 1.0) * f32(BIB_ROWS - 1u);
    let column = cage_rem_tau(angle) / TAU * f32(BIB_COLUMNS - 1u);
    let r = u32(floor(row));
    let c = u32(floor(column));
    let r1 = min(r + 1u, BIB_ROWS - 1u);
    let c1 = min(c + 1u, BIB_COLUMNS - 1u);
    let a = mix(cage_word(COLLAR + r * BIB_COLUMNS + c),
                cage_word(COLLAR + r * BIB_COLUMNS + c1), fract(column));
    let b = mix(cage_word(COLLAR + r1 * BIB_COLUMNS + c),
                cage_word(COLLAR + r1 * BIB_COLUMNS + c1), fract(column));
    return vec3<f32>(sin(angle), 0.0, cos(angle)) * mix(a, b, fract(row));
}

// A point on the collar, from its top plane (`t` zero) down to its base.
fn collar_point(t: f32, control: f32) -> vec3<f32> {
    let angle = surface_angle(control);
    let start_depth = cage_word(COLLAR_RADIUS + 1u);
    let end_depth = depth_at(cos(angle) >= 0.0, 0.0);
    let depth = start_depth + (end_depth - start_depth) * t;
    let collar_width = cage_word(COLLAR_RADIUS);
    let width = collar_width + (cage_word(BASE_RADIUS) - collar_width) * t;
    let radius = polar_radius(width, depth, collar_power(angle), angle);
    let z = cage_word(CENTER + 1u) + radius * cos(angle);
    let top = plane_at(0u, z);
    // Lower equipment seats the bib/collar seam; blend that displacement
    // through the collar while its anatomical top opening stays unchanged.
    let seating = bib_offset(0.0, angle) * t;
    return vec3<f32>(
        cage_word(CENTER) + radius * sin(angle),
        top + (base_height(angle, z) - top) * t,
        z,
    ) + seating + collar_offset(t, angle);
}

// A point on the bib, from the collar's base (`t` zero) down to the hem,
// seated by its measured offset.
fn bib_point(t: f32, control: f32) -> vec3<f32> {
    let hem = hem_height(control);
    let angle = surface_angle(control);
    let cosine = cos(angle);
    let front = cosine >= 0.0;
    let base_radius = cage_word(BASE_RADIUS);
    let outer = select(cage_word(OUTER_WIDTH + 1u), cage_word(OUTER_WIDTH), front);
    let side = base_radius + (min(cage_word(OUTER_WIDTH), cage_word(OUTER_WIDTH + 1u)) - base_radius);
    let sine2 = pow2(sin(angle));
    let width = base_radius + (outer - base_radius + (side - outer) * sine2) * t;
    let depth = depth_at(front, t);
    let hem_power = select(6.0, 2.0, front);
    let radius = polar_radius(
        width,
        depth,
        collar_power(angle) + (hem_power - collar_power(angle)) * t,
        angle,
    );
    // The authored chart descends smoothly before anatomical seating; its
    // shoulder crown is limited to the available collar-to-hem drop.
    let base_depth = depth_at(front, 0.0);
    let base_z = cage_word(CENTER + 1u)
        + polar_radius(base_radius, base_depth, collar_power(angle), angle) * cosine;
    let base = base_height(angle, base_z);
    let available_drop = base - hem;
    let shoulder_crown = min(cage_word(CROWN), available_drop * 0.8 / PI) * sine2 * sin(PI * t);
    let offset = bib_offset(t, angle) + collar_offset(1.0, angle) * (1.0 - t);
    return vec3<f32>(
        cage_word(CENTER) + radius * sin(angle) + offset.x,
        base + (hem - base) * t + shoulder_crown + offset.y,
        cage_word(CENTER + 1u) + radius * cosine + offset.z,
    );
}

fn gorget_collar(t: f32, angle: f32) -> vec3<f32> {
    return collar_point(t, angle);
}

fn gorget_bib(t: f32, angle: f32) -> vec3<f32> {
    return bib_point(t, angle);
}

fn gorget_center() -> vec2<f32> {
    return vec2<f32>(cage_word(CENTER), cage_word(CENTER + 1u));
}
"#;
