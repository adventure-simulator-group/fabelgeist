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
        // A lower sleeve's torso branch must not inflate its armhole fit.
        // Ownership was filtered on the host; this spatial test uses the
        // fitted device frame, before any triangle is clipped into a band.
        if (face >= params.body_face_count) {
            let layer_support_envelope_scale = 1.8;
            let radial_limit = max(f.half_extents.x, f.half_extents.z) * layer_support_envelope_scale;
            var nearby = false;
            for (var corner = 0u; corner < 3u; corner++) {
                let p = host_local(f, positions_at(faces[face * 3u + corner]));
                nearby = nearby || (p.y >= extent.x - HALF_WIDTH && p.y <= extent.y + HALF_WIDTH && length(p.xz) <= radial_limit);
            }
            if (!nearby) { continue; }
        }
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
    let vertices = build_hull(first, count);
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
        let radius = radius_from(first, vertices, center, direction);
        sections[output + 2u + r] = radius;
    }
}
