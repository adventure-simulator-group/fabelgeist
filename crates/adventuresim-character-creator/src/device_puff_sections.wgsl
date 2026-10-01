// Independent stations collect the same triangle fragments as the original
// fitter, then build its sorted monotone-chain hull. No vertex-density approximation.
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let section = id.x;
    let f = frame_at(0u);
    let extent = span(f, params.inset);
    let y = mix(extent.x, extent.y, f32(section) / f32(SECTIONS - 1u));
    let bounds = vec2<f32>(y - HALF_WIDTH, y + HALF_WIDTH);
    let first = section * params.capacity;
    var count = 0u;
    for (var face = 0u; face < params.face_count; face++) {
        for (var corner = 0u; corner < 3u; corner++) {
            let a = host_local(f, positions_at(faces[face * 3u + corner]));
            let b = host_local(f, positions_at(faces[face * 3u + (corner + 1u) % 3u]));
            if (a.y >= bounds.x && a.y <= bounds.y) {
                sample_set(first + count, a.xz); count++;
            }
            for (var plane = 0u; plane < 2u; plane++) {
                let height = bounds[plane];
                if ((a.y < height && b.y > height) || (b.y < height && a.y > height)) {
                    sample_set(first + count, mix(a.xz, b.xz, (height - a.y) / (b.y - a.y))); count++;
                }
            }
        }
    }
    sections[section * SECTION_WORDS] = f32(count);
    if (count < 3u) { sections[section * SECTION_WORDS + 2u] = -1.0; atomicOr(&status[0], 1u); return; }
    // Sorting gives the hull a strictly advancing order even when adjacent
    // triangle slices produce almost collinear points. Gift wrapping can
    // cycle on those points after f32 orientation roundoff.
    for (var i = 1u; i < count; i++) {
        let p = sample_at(first + i);
        var at = i;
        while (at > 0u) {
            let previous = sample_at(first + at - 1u);
            if (previous.x < p.x || (previous.x == p.x && previous.y <= p.y)) { break; }
            sample_set(first + at, previous); at--;
        }
        sample_set(first + at, p);
    }
    var unique = 1u;
    for (var i = 1u; i < count; i++) {
        let p = sample_at(first + i);
        if (any(p != sample_at(first + unique - 1u))) {
            sample_set(first + unique, p); unique++;
        }
    }
    var vertices = 0u;
    for (var chain = 0u; chain < 2u; chain++) {
        let chain_start = vertices;
        for (var i = 0u; i < unique; i++) {
            let at = select(i, unique - 1u - i, chain == 1u);
            let p = sample_at(first + at);
            while (vertices >= chain_start + 2u) {
                let a = hull_at(first + vertices - 2u);
                let b = hull_at(first + vertices - 1u);
                if (cross2(b - a, p - b) > 0.0) { break; }
                vertices--;
            }
            hull_set(first + vertices, p); vertices++;
        }
        vertices--;
    }
    sections[section * SECTION_WORDS + 1u] = f32(vertices);
    if (vertices < 3u) { sections[section * SECTION_WORDS + 2u] = -2.0; atomicOr(&status[0], 1u); return; }
    let start = hull_at(first);
    // Translation before accumulation keeps precision on off-center sections.
    var area = 0.0;
    var moment = vec2<f32>(0.0);
    for (var i = 0u; i < vertices; i++) {
        let a = hull_at(first + i) - start;
        let b = hull_at(first + (i + 1u) % vertices) - start;
        let signed = cross2(a, b);
        area += signed; moment += (a + b) * signed;
    }
    if (!(abs(area) > 1e-12)) { sections[section * SECTION_WORDS + 2u] = -3.0; atomicOr(&status[0], 1u); return; }
    let center = start + moment / (3.0 * area);
    let output = section * SECTION_WORDS;
    sections[output] = center.x; sections[output + 1u] = center.y;
    for (var r = 0u; r < RADII; r++) {
        let angle = f32(r) / f32(RADII) * TAU;
        let direction = vec2<f32>(cos(angle), sin(angle));
        // Original section fitter's minimum radius, in metres.
        var radius = 0.012;
        for (var i = 0u; i < vertices; i++) {
            let a = hull_at(first + i) - center;
            let edge = hull_at(first + (i + 1u) % vertices) - hull_at(first + i);
            let determinant = cross2(direction, edge);
            if (abs(determinant) < 1e-8) { continue; }
            let distance = cross2(a, edge) / determinant;
            let along = cross2(a, direction) / determinant;
            if (along >= 0.0 && along <= 1.0 && distance > radius) { radius = distance; }
        }
        sections[output + 2u + r] = radius;
    }
}
