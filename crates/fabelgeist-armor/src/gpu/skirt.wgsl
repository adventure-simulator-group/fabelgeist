// Native design words; shape controls are independent of the fitted enclosure.
const PROFILE: u32 = 0u;
const HEM_WIDTH: u32 = 1u;
const HEM_DEPTH: u32 = 2u;
const ROUNDNESS: u32 = 3u;
const FITTED_FRACTION: u32 = 4u;
const FIRST_SKIRT_ROW: u32 = 5u;
const MINIMUM_JOIN_RADIAL_NORMAL: f32 = 0.2;

fn section(row: u32) -> u32 {
    return FIRST_SECTION + row * SECTION_WORDS;
}

fn radius(row: u32, column: u32) -> f32 {
    return frames[section(row) + column];
}

fn skirt_point(column: u32, row: u32) -> vec3<f32> {
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
    return vec3<f32>(point.x, fit.half_extents.y * (1.0 - 2.0 * descent), point.y);
}

fn attachment_vector(point: u32, offset: u32) -> vec3<f32> {
    let first = ATTACHMENT + point * POINT_WORDS + offset;
    return vec3<f32>(frames[first], frames[first + 1u], frames[first + 2u]);
}

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    if (coord.w < 0.5) { return ShellVertex(skirt_point(u32(coord.x), u32(coord.y)), 0.0); }
    let body = attachment_vector(u32(coord.x), 0u);
    if (coord.w < 1.5) { return ShellVertex(body, 0.0); }
    let column = u32(coord.y);
    let row = u32(design[FIRST_SKIRT_ROW]);
    let lower = skirt_point(column, row);
    let normal = attachment_vector(u32(coord.x), 3u);
    let angle = TAU * f32(column) / f32(COLUMNS);
    let radial = vec2<f32>(sin(angle), cos(angle));
    let height = body.y - lower.y;
    let slope = normal.y / max(dot(normal.xz, radial), MINIMUM_JOIN_RADIAL_NORMAL);
    let top_tangent = vec3<f32>(radial.x * slope * height, -height, radial.y * slope * height);
    let bottom_tangent = (skirt_point(column, row + 1u) - lower)
        * (height / (2.0 * fit.half_extents.y / f32(ROWS)));
    let t = coord.z;
    let t2 = t * t;
    let t3 = t2 * t;
    let point = body * (2.0 * t3 - 3.0 * t2 + 1.0)
        + top_tangent * (t3 - 2.0 * t2 + t)
        + lower * (-2.0 * t3 + 3.0 * t2)
        + bottom_tangent * (t3 - t2);
    return ShellVertex(point, 0.0);
}
