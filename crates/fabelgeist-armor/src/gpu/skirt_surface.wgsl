// Native design words; shape controls are independent of the fitted enclosure.
const PROFILE: u32 = 0u;
const HEM_WIDTH: u32 = 1u;
const HEM_DEPTH: u32 = 2u;
const ROUNDNESS: u32 = 3u;
const FITTED_FRACTION: u32 = 4u;
const FIRST_SKIRT_ROW: u32 = 5u;
const PLEAT_COUNT: u32 = 6u;
const PLEAT_DEPTH: u32 = 7u;
const PLEAT_STYLE: u32 = 8u;
const PLEAT_ROTATION: u32 = 9u;
const MINIMUM_JOIN_RADIAL_NORMAL: f32 = 0.2;

fn section(row: u32) -> u32 {
    return FIRST_SECTION + row * SECTION_WORDS;
}

// Continuous lattice sampling also seats arbitrary support strip counts.
fn angular_radius(row: u32, column: f32) -> f32 {
    let wrapped = column - floor(column / f32(COLUMNS)) * f32(COLUMNS);
    let first = u32(floor(wrapped));
    return mix(frames[section(row) + first], frames[section(row) + (first + 1u) % COLUMNS], fract(wrapped));
}
fn radius(row: f32, column: f32) -> f32 {
    let bounded = clamp(row, 0.0, f32(ROWS));
    return mix(angular_radius(u32(floor(bounded)), column), angular_radius(min(u32(floor(bounded)) + 1u, ROWS), column), fract(bounded));
}

fn skirt_surface(column: f32, row: f32, fold_share: f32) -> vec3<f32> {
    let descent = f32(row) / f32(ROWS);
    let angle = TAU * f32(column) / f32(COLUMNS);
    let direction = vec2<f32>(sin(angle), cos(angle));
    let release = smoothstep(0.0, design[FITTED_FRACTION], descent);
    let required = radius(row, column);
    var hanging = 0.0;
    for (var sample = 0u; sample <= ROWS; sample++) {
        hanging = max(hanging, radius(f32(sample), column));
    }
    let t = clamp((descent - design[FITTED_FRACTION]) / (1.0 - design[FITTED_FRACTION]), 0.0, 1.0);
    var progress = t;
    if (design[PROFILE] > 1.5) {
        // Quintic release keeps a tangent join even at the smallest permitted
        // bell exponent, where a linear release would create a sharp shoulder.
        let release_t = t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
        progress = pow(sin(release_t * PI * 0.5), design[ROUNDNESS]);
    }
    let width = mix(1.0, design[HEM_WIDTH], progress);
    let depth = mix(1.0, design[HEM_DEPTH], progress);
    let gain = width * depth / length(vec2<f32>(depth * direction.x, width * direction.y));
    let shaped = mix(required, hanging * gain, release);
    // The fitted hull already includes the normal clearance and wall gauge.
    let phase = fract(column / f32(COLUMNS) * design[PLEAT_COUNT] + design[PLEAT_ROTATION]);
    let rounded = 0.5 - 0.5 * cos(TAU * phase);
    let accordion = 1.0 - abs(2.0 * phase - 1.0);
    let pleat = mix(rounded, accordion, design[PLEAT_STYLE]) * design[PLEAT_DEPTH] * release;
    let r = max(required, shaped) + pleat * fold_share;
    let point = direction * r;
    return vec3<f32>(point.x, fit.half_extents.y * (1.0 - 2.0 * descent), point.y);
}

fn skirt_point(column: f32, row: f32) -> vec3<f32> {
    return skirt_surface(column, row, 1.0);
}

