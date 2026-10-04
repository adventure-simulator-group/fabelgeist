fn section_radius(section: u32, direction: vec2<f32>) -> f32 {
    let coordinate = rem_tau(atan2(direction.y, direction.x)) / TAU * f32(RADII);
    let index = u32(floor(coordinate));
    let t = fract(coordinate);
    let t2 = t * t;
    let t3 = t2 * t;
    let weights = vec4<f32>((1.0 - t) * (1.0 - t) * (1.0 - t),
        3.0 * t3 - 6.0 * t2 + 4.0, -3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0, t3) / 6.0;
    var radius = 0.0;
    for (var k = 0u; k < 4u; k++) {
        radius += weights[k] * sections[section * SECTION_WORDS + 2u + (index + k + RADII - 1u) % RADII];
    }
    return radius;
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count || atomicLoad(&status[0]) != 0u) { return; }
    let f = frame_at(0u);
    let authored = span(f, 0.0);
    let fitted = span(f, params.inset);
    var p = host_local(f, carriers_at(i));
    let axial = clamp((p.y - authored.x) / (authored.y - authored.x), 0.0, 1.0);
    p.y = mix(fitted.x, fitted.y, axial);
    let station = axial * f32(SECTIONS - 1u);
    let first = min(u32(floor(station)), SECTIONS - 2u);
    let t = station - f32(first);
    let blend = t * t * (3.0 - 2.0 * t);
    let a = first * SECTION_WORDS;
    let b = (first + 1u) * SECTION_WORDS;
    let center = mix(vec2<f32>(sections[a], sections[a + 1u]), vec2<f32>(sections[b], sections[b + 1u]), blend);
    let distance = length(p.xz);
    let direction = p.xz / max(distance, FLT_EPSILON);
    let ellipse_radius = 1.0 / length(direction / f.half_extents.xz);
    var allowance = distance - ellipse_radius;
    if (params.taper != 0u) {
        let transition = clamp((fitted.y - p.y) / ((fitted.y - fitted.x) / params.puffs * TAPER_COURSES), 0.0, 1.0);
        allowance = params.allowance + max(allowance - params.allowance, 0.0) * transition;
    }
    let radius = mix(section_radius(first, direction), section_radius(first + 1u, direction), blend) + allowance;
    let radial = center + direction * radius;
    carriers_set(i, host_point(f, vec3<f32>(radial.x, p.y, radial.y)));
}
