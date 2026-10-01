// Convex sections shared by clothing and rigid thigh carriers.
fn cross2(a: vec2<f32>, b: vec2<f32>) -> f32 { return a.x * b.y - a.y * b.x; }
fn sample_at(at: u32) -> vec2<f32> { return vec2<f32>(samples[at * 2u], samples[at * 2u + 1u]); }
fn sample_set(at: u32, p: vec2<f32>) { samples[at * 2u] = p.x; samples[at * 2u + 1u] = p.y; }
fn hull_at(at: u32) -> vec2<f32> { return vec2<f32>(hulls[at * 2u], hulls[at * 2u + 1u]); }
fn hull_set(at: u32, p: vec2<f32>) { hulls[at * 2u] = p.x; hulls[at * 2u + 1u] = p.y; }
fn build_hull(first: u32, count: u32) -> u32 {
    if (count < 3u) { return 0u; }
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
    return vertices;
}
fn radius_from(first: u32, vertices: u32, center: vec2<f32>, direction: vec2<f32>) -> f32 {
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
    return radius;
}
