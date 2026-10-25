// Exact horizontal sections of the anatomical support triangles. Sparse
// bodies need not have vertices at a query's height or inside its lateral band.
struct Section {
    a: vec3<f32>,
    b: vec3<f32>,
    valid: bool,
}

fn section(face: u32, height: f32) -> Section {
    var ends: array<vec3<f32>, 2>;
    var count = 0u;
    for (var edge = 0u; edge < 3u; edge += 1u) {
        let a = local(positions_at(support[face * 3u + edge]));
        let b = local(positions_at(support[face * 3u + (edge + 1u) % 3u]));
        if ((a.y > height) != (b.y > height)) {
            ends[count] = a + (b - a) * ((height - a.y) / (b.y - a.y));
            count += 1u;
        }
    }
    return Section(ends[0], ends[1], count == 2u);
}

// Clip a section to each lateral band and reduce its outward depth. A single
// intersecting segment establishes surface support, unlike nearby vertices.
fn reduce_depth(s: Section, across: f32, center: f32, facing: f32, at: u32) {
    if (!s.valid) { return; }
    let delta = s.b - s.a;
    for (var side = 0u; side < 2u; side += 1u) {
        let x = select(-across, across, side == 1u);
        var first = 0.0;
        var last = 1.0;
        if (delta.x == 0.0) {
            if (abs(s.a.x - x) > TRANSVERSE_BAND_HALF_WIDTH_M) { continue; }
        } else {
            let left = (x - TRANSVERSE_BAND_HALF_WIDTH_M - s.a.x) / delta.x;
            let right = (x + TRANSVERSE_BAND_HALF_WIDTH_M - s.a.x) / delta.x;
            first = max(first, min(left, right));
            last = min(last, max(left, right));
            if (first > last) { continue; }
        }
        let depth = max((s.a.z + delta.z * first) * facing,
                        (s.a.z + delta.z * last) * facing);
        if (depth <= center * facing) { continue; }
        atomicMax(&work[at], ordered_from_float(depth));
        count_at(at);
    }
}
