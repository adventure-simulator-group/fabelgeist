const WALL: u32 = 0u;
const CLEARANCE: u32 = 1u;
const ARM_ALLOWANCE: u32 = 2u;
const FRONT_REACH: u32 = 3u;
const REAR_REACH: u32 = 4u;
const FRONT_DROP: u32 = 5u;
const REAR_DROP: u32 = 6u;
const NECK_REACH: u32 = 7u;
const CROWN: u32 = 8u;
const ARM_LENGTH: u32 = 9u;
const LOWER_LAMES: u32 = 10u;
const FLUTE_DEPTH: u32 = 11u;
const FRONT_RETURN: u32 = 12u;
const REAR_RETURN: u32 = 13u;
const FRONT_EXTENSION: u32 = 14u;
const REAR_EXTENSION: u32 = 15u;
const WING_START: u32 = 16u;
const CORNER_ROUNDING: u32 = 17u;
const FRONT_POSITION: u32 = 18u;
const REAR_POSITION: u32 = 19u;
const FRONT_ROUNDING: u32 = 20u;
const REAR_ROUNDING: u32 = 21u;
const UPPER_SPAN: u32 = 22u;
const ARM_WRAP: u32 = 23u;

const WIDTH_TAPER: f32 = 0.014;
const HEIGHT_TAPER: f32 = 0.010;
const DELTOID_JOIN: f32 = 0.040;
const NECK_OVERLAP: f32 = 0.02;

fn saddle_width() -> f32 {
    return max(fit.half_extents.x + design[CLEARANCE] + design[WALL] + design[ARM_ALLOWANCE],
        fit.half_extents.x + design[WALL] + WIDTH_TAPER + 0.002);
}
fn saddle_height() -> f32 {
    return max(fit.half_extents.z * design[CROWN],
        fit.half_extents.z + design[CLEARANCE] + design[WALL] + HEIGHT_TAPER);
}
fn lap_step(taper: f32) -> f32 {
    return max(taper / design[LOWER_LAMES], design[WALL] * 1.25 + design[FLUTE_DEPTH]);
}
fn lap_reserve(taper: f32) -> f32 {
    return (lap_step(taper) - taper / design[LOWER_LAMES]) * (design[LOWER_LAMES] - 1.0);
}
fn rounded_end(distance: f32, span: f32) -> f32 {
    let t = clamp(distance / span, 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}
fn saddle_theta(u: f32, v: f32) -> f32 {
    let main_end = 1.0 - design[UPPER_SPAN] + NECK_OVERLAP;
    let corner = rounded_end(v, main_end * design[CORNER_ROUNDING]);
    let front_sign = sign(fit.x.z);
    let rear = mix(design[ARM_WRAP], select(design[FRONT_RETURN], design[REAR_RETURN], front_sign > 0.0), corner);
    let front = mix(design[ARM_WRAP], select(design[REAR_RETURN], design[FRONT_RETURN], front_sign > 0.0), corner);
    return mix(-rear, front, u);
}
fn wing_drop(theta: f32, v: f32) -> f32 {
    let anterior = theta * sign(fit.x.z) > 0.0;
    let wing = select(design[REAR_EXTENSION], design[FRONT_EXTENSION], anterior);
    let edge = select(design[REAR_RETURN], design[FRONT_RETURN], anterior);
    let fraction = clamp((abs(theta) - design[WING_START]) / (edge - design[WING_START]), 0.0, 1.0);
    let position = select(design[REAR_POSITION], design[FRONT_POSITION], anterior);
    let rounding = select(design[REAR_ROUNDING], design[FRONT_ROUNDING], anterior);
    let radius = min(position, 1.0 - position) * (1.0 - rounding);
    let along = v / (1.0 - design[UPPER_SPAN] + NECK_OVERLAP);
    let hanging = rounded_end(along, position - radius) * rounded_end(1.0 - along, 1.0 - position - radius);
    return wing * fraction * fraction * (3.0 - 2.0 * fraction) * hanging;
}
fn saddle_point(u: f32, v: f32) -> vec3<f32> {
    let horizontal = normalize(vec3<f32>(fit.y.x, 0.0, fit.y.z));
    let medial = vec3<f32>(dot(fit.x, horizontal), dot(fit.y, horizontal), dot(fit.z, horizontal));
    let up = vec3<f32>(fit.x.y, fit.y.y, fit.z.y);
    let theta = saddle_theta(u, v);
    let anterior = theta * sign(fit.x.z) > 0.0;
    let reach = select(design[REAR_REACH], design[FRONT_REACH], anterior);
    let drop = select(design[REAR_DROP], design[FRONT_DROP], anterior);
    let inward = design[NECK_REACH] + reach * pow2(sin(min(abs(theta), PI * 0.5)));
    let gap = max(design[WALL] * 2.0 + design[FLUTE_DEPTH], 0.006);
    let width = saddle_width() + gap + lap_reserve(WIDTH_TAPER);
    let transverse = select(sign(sin(theta)) * (1.0 - 0.08 * -cos(theta)), sin(theta), cos(theta) >= 0.0);
    let inner_x = width * transverse * 1.1;
    let inner_z = saddle_height() * cos(theta) - drop * max(-cos(theta), 0.0);
    let crown = max(saddle_height() * 1.1, saddle_height() * 1.04 + gap)
        + design[ARM_ALLOWANCE] + lap_reserve(HEIGHT_TAPER);
    let shift = crown * (cos(min(abs(theta), min(design[ARM_WRAP], PI * 0.5))) - cos(theta)) * medial.z;
    let outer = vec3<f32>(width * sin(theta), -DELTOID_JOIN, crown * cos(theta)) + medial * shift;
    let inner = vec3<f32>(inner_x, 0.0, 0.0) + medial * inward + up * inner_z;
    return mix(outer, inner, v) + up * (0.012 * sin(PI * v) - wing_drop(theta, v));
}
fn arm_lame(u: f32, v: f32, index: f32) -> vec3<f32> {
    let theta = design[ARM_WRAP] * (2.0 * u - 1.0);
    let width = saddle_width() + lap_reserve(WIDTH_TAPER) - lap_step(WIDTH_TAPER) * index;
    let height = saddle_height() * 1.04 + design[ARM_ALLOWANCE] + lap_reserve(HEIGHT_TAPER) - lap_step(HEIGHT_TAPER) * index;
    let step = design[ARM_LENGTH] / design[LOWER_LAMES];
    return vec3<f32>(sin(theta) * width, -DELTOID_JOIN - ((index + 1.0) * step - v * (step + 0.004)), cos(theta) * height);
}
