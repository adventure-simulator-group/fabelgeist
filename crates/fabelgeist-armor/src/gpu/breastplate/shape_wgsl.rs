//! The breastplate's authored shape in WGSL: the reference sections, their
//! cubic interpolation, the chart's boundaries and the carrier point.
//!
//! Every function reads `plate`, the design's words followed by the
//! wearer's, which the wearer kernels complete on the device. Reference
//! tables are typed `f32` constants so that expressions over them round in
//! `f32`.

use crate::BreastplateDesign;

/// Words of the design at the start of the plate buffer.
pub(crate) const DESIGN_WORDS: u32 = 32;
/// Words of the wearer after the design.
pub(crate) const WEARER_WORDS: u32 = 32;

/// The design as the kernels read it; the order is `SHAPE`'s accessors.
pub(crate) fn design_words(design: &BreastplateDesign) -> Vec<f32> {
    let profile = &design.profile;
    let mut words = vec![
        design.plate_length.unit(),
        design.neck_depth.unit(),
        design.arm_opening_depth.unit(),
        design.neck_width.unit(),
        design.side_return.unit(),
        design.back_depth.unit(),
        design.waist_width.unit(),
        design.skirt_length.unit(),
        design.skirt_flare.metres(),
        design.front_clearance.metres(),
        design.back_clearance.metres(),
        design.wall_thickness.metres(),
        profile.projection.metres(),
        profile.projection_height.unit(),
        profile.fullness.unit(),
        profile.medial_ridge.metres(),
        profile.waist_point.metres(),
        profile.waist_point_width.unit(),
        profile.waist_projection.metres(),
        profile.upper_chest_recession.metres(),
        f32::from(u8::from(design.fluting.is_some())),
    ];
    words.extend(design.fluting.as_ref().map_or([0.0; 8], |f| {
        [
            f32::from(f.count.0),
            f.width.unit(),
            f.depth.metres(),
            f.spread.unit(),
            f.lower_spread.unit(),
            f.start.unit(),
            f.end.unit(),
            f.fade.unit(),
        ]
    }));
    words.resize((DESIGN_WORDS + WEARER_WORDS) as usize, 0.0);
    words
}

/// The normalization every breastplate kernel has; its arithmetic, like the
/// shape's and the fit's, rounds every step in a fixed order with exact
/// device arithmetic (see [`fabelgeist_compute::host_float`]), so that the
/// plate is deterministic and independent of the device's fused operations.
pub(crate) const UNIT: &str = r#"
// The vector scaled to unit length: w is one when the vector could be.
fn unit(a: vec3<f32>) -> vec4<f32> {
    let magnitude = host_length(a);
    if (magnitude > 1e-12 && is_finite(magnitude)) {
        return vec4<f32>(host_scale3(a, host_div(1.0, magnitude)), 1.0);
    }
    return vec4<f32>(a, 0.0);
}
"#;

/// Design and wearer accessors, the frame, and the authored carrier.
pub(crate) const SHAPE: &str = r#"
const REFERENCE_RIG_NECK_HEIGHT: f32 = 1.4419107;
const REFERENCE_CARRIER_TOP_HEIGHT: f32 = 1.480;
const REFERENCE_SEMANTIC_HEIGHT: f32 = 0.502445;
const REFERENCE_SHOULDER_HALF_WIDTH: f32 = 0.17586103;
const REFERENCE_TORSO_HALF_WIDTH: f32 = 0.18716274;
const REFERENCE_SECTION_CENTER_DEPTH: f32 = 0.02030905;
const REFERENCE_SECTION_RADIUS: f32 = 0.12175832;
// Rust's `f32::to_radians` factor.
const RADIANS_PER_DEGREE: f32 = 0.0174532924;

const FRONT_HEIGHTS = array<f32, 10>(1.038, 1.105, 1.185, 1.285, 1.355, 1.400, 1.445, 1.480, 0.0, 0.0);
const FRONT_RADIUS_X = array<f32, 10>(0.178, 0.180, 0.181, 0.183, 0.185, 0.174, 0.142, 0.132, 0.0, 0.0);
const FRONT_RADIUS_Z = array<f32, 10>(0.143, 0.153, 0.156, 0.151, 0.136, 0.115, 0.080, 0.050, 0.0, 0.0);
const FRONT_TRIM_HEIGHTS = array<f32, 10>(
    1.038, 1.105, 1.185, 1.250, 1.300, 1.335, 1.370, 1.400, 1.445, 1.480
);
const FRONT_LIMIT_DEGREES = array<f32, 10>(92.0, 94.0, 94.0, 82.0, 68.0, 60.0, 52.0, 60.0, 72.0, 96.0);
const FRONT_NECK_Y = array<f32, 10>(1.425, 1.433, 1.458, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
const BACK_HEIGHTS = array<f32, 10>(1.040, 1.120, 1.200, 1.280, 1.340, 1.400, 1.445, 1.480, 0.0, 0.0);
const BACK_RADIUS_X = array<f32, 10>(0.168, 0.170, 0.180, 0.195, 0.210, 0.205, 0.170, 0.150, 0.0, 0.0);
const BACK_RADIUS_Z = array<f32, 10>(0.080, 0.086, 0.092, 0.097, 0.099, 0.098, 0.088, 0.078, 0.0, 0.0);
const BACK_TRIM_HEIGHTS = array<f32, 10>(1.040, 1.120, 1.200, 1.280, 1.340, 1.380, 1.420, 1.460, 1.480, 0.0);
const BACK_LIMIT_DEGREES = array<f32, 10>(89.0, 89.0, 74.0, 54.0, 46.0, 48.0, 46.0, 65.0, 70.0, 0.0);
const BACK_NECK_Y = array<f32, 10>(1.420, 1.430, 1.455, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
const NECK_U = array<f32, 10>(0.0, 0.25, 0.50, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

const WAIST_POINT_BLEND_HEIGHT: f32 = 0.45;
const RIDGE_FADE_START: f32 = 0.65;
const UPPER_CHEST_PHASE: f32 = 0.65;
const BACK_UPPER_SECTION_START: f32 = 1.20;
const BACK_UPPER_SECTION_BLEND: f32 = 0.08;
const BACK_UPPER_SECTION_POWER: f32 = 0.55;
const MAX_REAR_RETURN_DEGREES: f32 = 89.0;

fn plate_length() -> f32 { return plate[0]; }
fn neck_depth() -> f32 { return plate[1]; }
fn arm_opening_depth() -> f32 { return plate[2]; }
fn neck_width() -> f32 { return plate[3]; }
fn side_return() -> f32 { return plate[4]; }
fn back_depth() -> f32 { return plate[5]; }
fn waist_width() -> f32 { return plate[6]; }
fn skirt_length() -> f32 { return plate[7]; }
fn skirt_flare() -> f32 { return plate[8]; }
fn plate_clearance(rear: bool) -> f32 { return select(plate[9], plate[10], rear); }
fn wall_thickness() -> f32 { return plate[11]; }
fn projection() -> f32 { return plate[12]; }
fn projection_height() -> f32 { return plate[13]; }
fn fullness() -> f32 { return plate[14]; }
fn medial_ridge() -> f32 { return plate[15]; }
fn waist_point() -> f32 { return plate[16]; }
fn waist_point_width() -> f32 { return plate[17]; }
fn waist_projection() -> f32 { return plate[18]; }
fn upper_chest_recession() -> f32 { return plate[19]; }
fn fluted() -> bool { return plate[20] != 0.0; }
fn flute_count() -> f32 { return plate[21]; }
fn flute_width() -> f32 { return plate[22]; }
fn flute_depth() -> f32 { return plate[23]; }
fn flute_spread() -> f32 { return plate[24]; }
fn flute_lower_spread() -> f32 { return plate[25]; }
fn flute_start() -> f32 { return plate[26]; }
fn flute_end() -> f32 { return plate[27]; }
fn flute_fade() -> f32 { return plate[28]; }

// The wearer, after the design: its frame's lateral and front axes (the
// vertical is model up), the neck in that frame, and its scales.
const WEARER: u32 = 32u;
fn frame_lateral() -> vec3<f32> { return vec3<f32>(plate[WEARER], plate[WEARER + 1u], plate[WEARER + 2u]); }
fn frame_front() -> vec3<f32> {
    return vec3<f32>(plate[WEARER + 3u], plate[WEARER + 4u], plate[WEARER + 5u]);
}
fn neck_local() -> vec3<f32> { return vec3<f32>(plate[WEARER + 6u], plate[WEARER + 7u], plate[WEARER + 8u]); }
fn x_scale() -> f32 { return plate[WEARER + 9u]; }
fn shoulder_x_scale() -> f32 { return plate[WEARER + 10u]; }
fn y_scale() -> f32 { return plate[WEARER + 11u]; }
fn z_scale() -> f32 { return plate[WEARER + 12u]; }
fn lateral_origin() -> f32 { return plate[WEARER + 13u]; }
fn coronal_origin() -> f32 { return plate[WEARER + 14u]; }


fn local(point: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        host_dot(point, frame_lateral()),
        host_dot(point, vec3<f32>(0.0, 1.0, 0.0)),
        host_dot(point, frame_front()),
    );
}

fn world(point: vec3<f32>) -> vec3<f32> {
    return host_add3(
        host_add3(host_scale3(frame_lateral(), point.x), host_scale3(vec3<f32>(0.0, 1.0, 0.0), point.y)),
        host_scale3(frame_front(), point.z),
    );
}

fn cubic_slope(xs: array<f32, 10>, ys: array<f32, 10>, n: u32, zero_start: bool, index: u32) -> f32 {
    var x = xs;
    var y = ys;
    if (index == 0u) {
        if (zero_start) {
            return 0.0;
        }
        return host_div(host_sub(y[1], y[0]), host_sub(x[1], x[0]));
    }
    if (index + 1u == n) {
        return host_div(host_sub(y[index], y[index - 1u]), host_sub(x[index], x[index - 1u]));
    }
    return host_div(host_sub(y[index + 1u], y[index - 1u]), host_sub(x[index + 1u], x[index - 1u]));
}

// A cubic Hermite spline with centred slopes through the first `n` knots.
fn cubic(at: f32, xs: array<f32, 10>, ys: array<f32, 10>, n: u32, zero_start: bool) -> f32 {
    var x = xs;
    var y = ys;
    if (at <= x[0]) {
        return y[0];
    }
    if (at >= x[n - 1u]) {
        return y[n - 1u];
    }
    var index = n - 2u;
    for (var i = 0u; i + 1u < n; i = i + 1u) {
        if (at <= x[i + 1u]) {
            index = i;
            break;
        }
    }
    let s0 = cubic_slope(xs, ys, n, zero_start, index);
    let s1 = cubic_slope(xs, ys, n, zero_start, index + 1u);
    let h = host_sub(x[index + 1u], x[index]);
    let t = host_div(host_sub(at, x[index]), h);
    let t2 = host_mul(t, t);
    let t3 = host_mul(t2, t);
    let a = host_add(host_sub(host_mul(2.0, t3), host_mul(3.0, t2)), 1.0);
    let b = host_add(host_sub(t3, host_mul(2.0, t2)), t);
    let c = host_add(host_mul(-2.0, t3), host_mul(3.0, t2));
    let d = host_sub(t3, t2);
    return host_add(
        host_add(host_add(host_mul(a, y[index]), host_mul(host_mul(b, h), s0)), host_mul(c, y[index + 1u])),
        host_mul(host_mul(d, h), s1),
    );
}

fn heights(rear: bool) -> array<f32, 10> {
    if (rear) {
        return BACK_HEIGHTS;
    }
    return FRONT_HEIGHTS;
}

fn radii_x(rear: bool) -> array<f32, 10> {
    if (rear) {
        return BACK_RADIUS_X;
    }
    return FRONT_RADIUS_X;
}

fn bottom_height(rear: bool) -> f32 {
    return select(FRONT_HEIGHTS[0], BACK_HEIGHTS[0], rear);
}

fn mapped_height(reference_y: f32) -> f32 {
    return host_add(
        neck_local().y,
        host_mul(host_mul(host_sub(reference_y, REFERENCE_RIG_NECK_HEIGHT), y_scale()), plate_length())
    );
}

fn lateral_scale(reference_y: f32) -> f32 {
    let shoulder_blend = host_smoothstep(host_div(host_sub(reference_y, 1.355), 0.100));
    return host_add(
        host_mul(x_scale(), host_sub(1.0, shoulder_blend)), host_mul(shoulder_x_scale(), shoulder_blend)
    );
}

fn mapped_point(reference: vec3<f32>) -> vec3<f32> {
    let vertical_t = clamp(
        host_div(
            host_sub(reference.y, FRONT_HEIGHTS[0]), host_sub(REFERENCE_RIG_NECK_HEIGHT, FRONT_HEIGHTS[0])
        ),
        0.0,
        1.0,
    );
    let waist_scale = host_add(
        1.0, host_mul(host_sub(waist_width(), 1.0), host_sub(1.0, host_smoothstep(vertical_t)))
    );
    return vec3<f32>(
        host_add(lateral_origin(), host_mul(host_mul(reference.x, lateral_scale(reference.y)), waist_scale)),
        mapped_height(reference.y),
        host_add(
            coronal_origin(), host_mul(host_sub(reference.z, REFERENCE_SECTION_CENTER_DEPTH), z_scale())
        ),
    );
}

fn top_y(rear: bool, u: f32) -> f32 {
    var neck_y = FRONT_NECK_Y;
    var attach_y: f32 = 1.390;
    if (rear) {
        neck_y = BACK_NECK_Y;
        attach_y = 1.375;
    }
    var neck: array<f32, 10>;
    for (var k = 0u; k < 3u; k = k + 1u) {
        neck[k] = host_sub(
            REFERENCE_RIG_NECK_HEIGHT, host_mul(host_sub(REFERENCE_RIG_NECK_HEIGHT, neck_y[k]), neck_depth())
        );
    }
    let attach = host_sub(
        REFERENCE_RIG_NECK_HEIGHT,
        host_mul(host_sub(REFERENCE_RIG_NECK_HEIGHT, attach_y), arm_opening_depth())
    );
    let a = abs(u);
    if (a <= 0.5) {
        return cubic(a, NECK_U, neck, 3u, true);
    }
    let s = host_sub(1.0, host_cos(host_mul(host_sub(a, 0.5), PI)));
    return host_add(host_mul(neck[2], host_sub(1.0, s)), host_mul(attach, s));
}

fn top_neck_scale(reference_y: f32, bottom: f32, top: f32) -> f32 {
    let blend = host_smoothstep(host_div(host_sub(reference_y, bottom), max(host_sub(top, bottom), 1e-6)));
    return host_add(1.0, host_mul(host_sub(neck_width(), 1.0), blend));
}

// The authored front or rear section at (theta, y), in reference space. Each
// transcendental function is evaluated once, at one call site: their exact evaluations
// are long, and a driver inlines every site.
fn raw(rear: bool, theta: f32, y: f32) -> vec3<f32> {
    let s = host_sin(theta);
    let c = host_cos(theta);
    var a: f32;
    var b: f32;
    var base: f32;
    var power: f32;
    var phase = 0.0;
    var crown = 0.0;
    var ridge = 0.0;
    if (rear) {
        if (c <= 0.0) {
            shape_failed = true;
        }
        a = cubic(y, BACK_HEIGHTS, BACK_RADIUS_X, 8u, false);
        b = host_mul(cubic(y, BACK_HEIGHTS, BACK_RADIUS_Z, 8u, false), back_depth());
        let upper = host_smoothstep(
            host_div(host_sub(y, BACK_UPPER_SECTION_START), BACK_UPPER_SECTION_BLEND)
        );
        power = host_add(1.0, host_mul(host_sub(BACK_UPPER_SECTION_POWER, 1.0), upper));
        base = c;
    } else {
        a = cubic(y, FRONT_HEIGHTS, FRONT_RADIUS_X, 8u, false);
        b = cubic(y, FRONT_HEIGHTS, FRONT_RADIUS_Z, 8u, false);
        phase = clamp(
            host_div(host_sub(y, FRONT_HEIGHTS[0]), host_sub(FRONT_NECK_Y[0], FRONT_HEIGHTS[0])), 0.0, 1.0
        );
        let peak = projection_height();
        var bell: f32;
        if (phase <= peak) {
            bell = host_smoothstep(host_div(phase, peak));
        } else {
            bell = host_smoothstep(host_div(host_sub(1.0, phase), host_sub(1.0, peak)));
        }
        let upper_bell = host_mul(
            host_smoothstep(host_div(phase, UPPER_CHEST_PHASE)),
            host_sub(
                1.0,
                host_smoothstep(
                    host_div(host_sub(phase, UPPER_CHEST_PHASE), host_sub(1.0, UPPER_CHEST_PHASE))
                )
            ),
        );
        crown = host_add(
            host_sub(host_mul(projection(), bell), host_mul(upper_chest_recession(), upper_bell)),
            host_mul(waist_projection(), host_sub(1.0, host_smoothstep(phase))),
        );
        ridge = host_mul(
            medial_ridge(),
            host_sub(
                1.0,
                host_smoothstep(host_div(host_sub(phase, RIDGE_FADE_START), host_sub(1.0, RIDGE_FADE_START)))
            ),
        );
        power = host_div(2.0, fullness());
        base = max(c, 0.0);
    }
    let lifted = host_pow(base, power);
    if (rear) {
        return vec3<f32>(host_mul(a, s), y, host_mul(-b, lifted));
    }
    let point_drop = host_mul(
        host_mul(waist_point(), host_sub(1.0, host_smoothstep(host_div(phase, WAIST_POINT_BLEND_HEIGHT)))),
        waist_point_weight_of(s),
    );
    let upper = host_smoothstep(host_div(host_sub(y, 1.445), 0.035));
    let recession = host_mul(0.050, upper);
    let lift = host_mul(0.040, upper);
    let depth = host_add(
        host_sub(
            host_add(
                host_add(host_mul(b, c), host_mul(crown, lifted)), host_mul(ridge, host_sub(1.0, abs(s)))
            ),
            recession
        ),
        host_mul(lift, host_mul(s, s)),
    );
    return vec3<f32>(host_mul(a, s), host_sub(y, point_drop), depth);
}

// How strongly the waist point drops a section at an angle, from its sine: one at the
// centre, falling to zero at the waist point's width.
fn waist_point_weight_of(sine: f32) -> f32 {
    return max(host_sub(1.0, host_div(abs(sine), waist_point_width())), 0.0);
}

var<private> shape_failed: bool;

struct CarrierPoint {
    point: vec3<f32>,
    normal: vec3<f32>,
    // The authored section's depth at the point itself.
    raw_depth: f32,
};

// A loop bound the compiler cannot know, so that a loop around a long
// evaluation stays one call site instead of being unrolled.
fn opaque(count: u32) -> u32 {
    return count + host_zero();
}

// A carrier point: the authored section, mapped onto the wearer and offset by the
// clearance along its finite-difference normal.
fn carrier_point(rear: bool, theta: f32, y: f32) -> CarrierPoint {
    var mapped: array<vec3<f32>, 3>;
    var raw_depth = 0.0;
    for (var k = 0u; k < opaque(3u); k = k + 1u) {
        var at_theta = theta;
        var at_y = y;
        if (k == 1u) {
            at_theta = host_add(theta, 0.0005);
        } else if (k == 2u) {
            at_y = host_add(y, 0.0002);
        }
        let section = raw(rear, at_theta, at_y);
        if (k == 0u) {
            raw_depth = section.z;
        }
        mapped[k] = mapped_point(section);
    }
    let center = mapped[0];
    let normalized = unit(host_cross(host_sub3(mapped[1], center), host_sub3(mapped[2], center)));
    if (normalized.w == 0.0) {
        shape_failed = true;
    }
    var normal = normalized.xyz;
    if (rear) {
        normal = -normal;
    }
    return CarrierPoint(
        world(host_add3(center, host_scale3(normal, plate_clearance(rear)))), world(normal), raw_depth
    );
}

fn limit_theta(rear: bool, u: f32, y: f32) -> f32 {
    if (rear) {
        let trim_y = host_add(1.240, host_mul(host_sub(y, 1.240), arm_opening_depth()));
        let limit = host_mul(cubic(trim_y, BACK_TRIM_HEIGHTS, BACK_LIMIT_DEGREES, 9u, false), side_return());
        let edge = min(
            host_mul(limit, top_neck_scale(y, BACK_HEIGHTS[0], REFERENCE_CARRIER_TOP_HEIGHT)),
            MAX_REAR_RETURN_DEGREES
        );
        return host_mul(host_mul(edge, u), RADIANS_PER_DEGREE);
    }
    let trim_y = host_add(1.250, host_mul(host_sub(y, 1.250), arm_opening_depth()));
    let limit = host_mul(cubic(trim_y, FRONT_TRIM_HEIGHTS, FRONT_LIMIT_DEGREES, 10u, false), side_return());
    return host_mul(
        host_mul(host_mul(limit, u), top_neck_scale(y, FRONT_HEIGHTS[0], REFERENCE_CARRIER_TOP_HEIGHT)),
        RADIANS_PER_DEGREE
    );
}

// The chart's angle at column `u` and height `y`: the side limit, with the upper
// armscye blended in as its own boundary.
fn chart_theta(rear: bool, u: f32, y: f32) -> f32 {
    let original = limit_theta(rear, u, y);
    if (abs(u) <= 0.5) {
        return original;
    }
    let hs = heights(rear);
    let rs = radii_x(rear);
    // The upper boundary at the neck, the arm, and this column.
    var tops: array<f32, 3>;
    for (var i = 0u; i < opaque(3u); i = i + 1u) {
        var at = u;
        if (i == 0u) {
            at = 0.5;
        } else if (i == 1u) {
            at = 1.0;
        }
        tops[i] = top_y(rear, at);
    }
    let neck_y = tops[0];
    let arm_y = tops[1];
    let boundary_y = tops[2];
    var angles = array<f32, 5>(
        limit_theta(rear, 0.5, neck_y),
        limit_theta(rear, 1.0, arm_y),
        host_mul(host_sub(abs(u), 0.5), PI),
        limit_theta(rear, 0.5, y),
        limit_theta(rear, 0.5, boundary_y),
    );
    var sines: array<f32, 5>;
    for (var i = 0u; i < opaque(5u); i = i + 1u) {
        sines[i] = host_sin(angles[i]);
    }
    let neck_x = host_mul(cubic(neck_y, hs, rs, 8u, false), sines[0]);
    let arm_x = host_mul(cubic(arm_y, hs, rs, 8u, false), sines[1]);
    let along = sines[2];
    let radius = cubic(y, hs, rs, 8u, false);
    let inner_x = host_mul(radius, sines[3]);
    let boundary_inner_x = host_mul(cubic(boundary_y, hs, rs, 8u, false), sines[4]);
    let x = host_add(
        host_add(neck_x, host_mul(host_sub(arm_x, neck_x), along)),
        host_mul(host_sub(inner_x, boundary_inner_x), host_sub(1.0, along))
    );
    let boundary = host_mul(host_asin(clamp(host_div(x, radius), 0.0, 0.995)), sign(u));
    let phase = clamp(host_div(host_sub(y, hs[0]), host_sub(boundary_y, hs[0])), 0.0, 1.0);
    let blend = host_smoothstep(host_div(host_sub(phase, 0.75), 0.25));
    return host_add(host_mul(original, host_sub(1.0, blend)), host_mul(boundary, blend));
}
"#;
