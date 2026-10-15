// Both disconnected leg sections contribute to one shared convex hull.
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let section = id.x;
    let f = frame_at(0u);
    let y = f.half_extents.y * (1.0 - 2.0 * f32(section) / f32(SECTIONS - 1u));
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
    if (count < 3u) { atomicOr(&status[0], 1u); return; }
    // A skirt encloses its suspension axis too. This keeps forward/backward
    // displaced ankle sections representable around the common waist axis.
    sample_set(first + count, vec2<f32>(0.0)); count++;
    let vertices = build_hull(first, count);
    if (vertices < 3u) { atomicOr(&status[0], 1u); return; }
    let output = FIRST_SECTION + section * SECTION_WORDS;
    for (var column = 0u; column < RADII; column++) {
        let angle = TAU * f32(column) / f32(RADII);
        let direction = vec2<f32>(sin(angle), cos(angle));
        fit[output + column] = offset_radius(first, vertices, direction);
    }
}

// Intersect the ray with the hull's outward-offset supporting lines. Adding
// the allowance to a radial distance would under-clear shallow hull corners.
fn offset_radius(first: u32, vertices: u32, direction: vec2<f32>) -> f32 {
    var radius = INFINITY;
    for (var edge = 0u; edge < vertices; edge++) {
        let a = hull_at(first + edge);
        let delta = hull_at(first + (edge + 1u) % vertices) - a;
        let normal = normalize(vec2<f32>(delta.y, -delta.x));
        let facing = dot(direction, normal);
        if (facing > FLT_EPSILON) {
            radius = min(radius, (dot(a, normal) + params.allowance) / facing);
        }
    }
    if (!(radius > 0.0 && radius < INFINITY)) { atomicOr(&status[0], 1u); }
    return radius;
}
