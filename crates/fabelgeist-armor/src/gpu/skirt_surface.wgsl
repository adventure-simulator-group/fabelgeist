// Native design words; shape controls are independent of the fitted enclosure.
const PROFILE: u32 = 0u;
const HEM_WIDTH: u32 = 1u;
const HEM_DEPTH: u32 = 2u;
const ROUNDNESS: u32 = 3u;
const FITTED_FRACTION: u32 = 4u;
const FIRST_SKIRT_ROW: u32 = 5u;
const MINIMUM_JOIN_RADIAL_NORMAL: f32 = 0.2;
const ATTACHMENT_POINT_COUNT: u32 = 21u;

// A shaped cloned-body waist is authoritative. Advect each material column
// from that actual boundary toward the common hem, sampling the enclosing
// body profile at the resulting physical height.
fn waist_drop(column: f32) -> f32 {
    let count = u32(design[ATTACHMENT_POINT_COUNT]);
    if (count == 0u) { return 0.0; }
    let wrapped = fract(column / f32(COLUMNS)) * f32(COLUMNS);
    let first = u32(floor(wrapped));
    let a = ATTACHMENT + (count + first) * POINT_WORDS + 1u;
    let b = ATTACHMENT + (count + (first + 1u) % COLUMNS) * POINT_WORDS + 1u;
    let y = mix(frames[a], frames[b], fract(wrapped));
    return max((fit.half_extents.y - y) / (2.0 * fit.half_extents.y), 0.0);
}

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
    let physical_descent = mix(waist_drop(column), 1.0, descent);
    let angle = TAU * f32(column) / f32(COLUMNS);
    let direction = vec2<f32>(sin(angle), cos(angle));
    let release = smoothstep(0.0, design[FITTED_FRACTION], descent);
    let required = radius(physical_descent * f32(ROWS), column);
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
    var shaped = mix(required, hanging * gain, release);
    if (design[PROFILE] > 0.5 && design[PROFILE] < 1.5) {
        // A flared cloth falls from its waist toward the hem. Starting the
        // flare only after a body-width tube creates an artificial hip shelf.
        let hem_gain = design[HEM_WIDTH] * design[HEM_DEPTH] /
            length(vec2<f32>(design[HEM_DEPTH] * direction.x, design[HEM_WIDTH] * direction.y));
        let fall = mix(radius(0.0, column), hanging * hem_gain, descent);
        shaped = mix(required, fall, release);
    }
    // The fitted hull already includes the normal clearance and wall gauge.
    let pleat = skirt_fold(column, descent, release);
    let r = max(required, shaped) + pleat * fold_share;
    let point = direction * r;
    return vec3<f32>(point.x, fit.half_extents.y * (1.0 - 2.0 * physical_descent), point.y);
}

fn skirt_point(column: f32, row: f32) -> vec3<f32> {
    return skirt_surface(column, row, 1.0);
}

