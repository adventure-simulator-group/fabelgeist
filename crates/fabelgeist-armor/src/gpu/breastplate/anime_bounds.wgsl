@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> original: array<f32>;
@group(0) @binding(2) var<storage, read> references: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read> design: array<f32>;
@group(0) @binding(4) var<storage, read_write> bounds: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(6) var<uniform> params: Params;
@group(0) @binding(7) var<storage, read> columns: array<f32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let side = id.x;
    let width = select(params.width, params.extra, side != 0u);
    let offset = select(0u, params.front_count, side != 0u);
    let slope = design[3u + side];
    var floor = -1e30;
    var ceiling = 1e30;
    var half_width = 0.0;
    for (var i = 0u; i < width * (V_SAMPLES + SKIRT_SAMPLES - 1u); i += 1u) {
        let point = local(original_at(references[offset + i].x));
        half_width = max(half_width, abs(point.x - lateral_origin()));
    }
    for (var c = 0u; c < width; c += 1u) {
        let lower = local(original_at(references[offset + (V_SAMPLES + SKIRT_SAMPLES - 2u) * width + c].x));
        let upper = local(original_at(references[offset + (V_SAMPLES - 1u) * width + c].x));
        floor = max(floor, lower.y - slope * abs(lower.x - lateral_origin()));
        let column_offset = select(0u, params.width, side != 0u);
        if (abs(columns[column_offset + c]) <= NECKLINE_SIDE_COLUMN) {
            ceiling = min(ceiling, upper.y - slope * abs(upper.x - lateral_origin()));
        }
    }
    let height = (ceiling - floor) * design[1u] / design[0u];
    // Adjacent overlapping courses must remain separated by at least their
    // actual metal gauge. Deep laps can span multiple exposed pitches; a
    // pitch/overlap ratio alone does not establish a physical collision.
    let separation = design[5u] * height / (height + design[2u]);
    if (!(height > 0.0) || !(separation > wall_thickness())) { fail(STATUS_INVALID_SURFACE); }
    // Check the metal extrusion in overlapping bands. Radial layer separation
    // is only a construction bound; actual closed-shell contacts are checked
    // independently after course resampling and triangulation.
    for (var row = 0u; row < V_SAMPLES + SKIRT_SAMPLES - 1u; row += 1u) {
        for (var c = 0u; c < width; c += 1u) {
            let reference = references[offset + row * width + c];
            let point = local(original_at(reference.x));
            let q = point.y - slope * abs(point.x - lateral_origin());
            var in_lap = false;
            for (var course = 1u; course <= u32(design[0u]); course += 1u) {
                let high = floor + height * f32(course);
                if (q >= max(floor, high - design[2u]) && q <= high) { in_lap = true; }
            }
            if (!in_lap) { continue; }
            let normal = unit(original_at(reference.y) - original_at(reference.x));
            if (normal.w == 0.0) {
                fail(STATUS_INVALID_SURFACE);
            }
        }
    }
    if (!(half_width > 0.0)) { fail(STATUS_INVALID_SURFACE); }
    bounds[side] = vec4<f32>(floor, height, half_width, 1.0);
}
