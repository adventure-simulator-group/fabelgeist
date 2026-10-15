// Native design words; shape controls are independent of the fitted enclosure.
const PROFILE: u32 = 0u;
const HEM_WIDTH: u32 = 1u;
const HEM_DEPTH: u32 = 2u;
const ROUNDNESS: u32 = 3u;
const FITTED_FRACTION: u32 = 4u;

fn section(row: u32) -> u32 {
    return FIRST_SECTION + row * SECTION_WORDS;
}

fn radius(row: u32, column: u32) -> f32 {
    return frames[section(row) + column];
}

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    let column = u32(coord.x);
    let row = u32(coord.y);
    let descent = f32(row) / f32(ROWS);
    let angle = TAU * f32(column) / f32(COLUMNS);
    let direction = vec2<f32>(sin(angle), cos(angle));
    let release = smoothstep(0.0, design[FITTED_FRACTION], descent);
    let required = radius(row, column);
    var hanging = 0.0;
    for (var sample = 0u; sample <= ROWS; sample++) {
        hanging = max(hanging, radius(sample, column));
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
    let r = max(required, shaped);
    let point = direction * r;
    return ShellVertex(vec3<f32>(point.x, fit.half_extents.y * (1.0 - 2.0 * descent), point.y), 0.0);
}
