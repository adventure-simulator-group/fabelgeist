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
// How squarely a place must face the body before a seam there begins to
// leave the cloth beneath, and before it has left it.
const SEAM_BESIDE_BODY: vec2<f32> = vec2<f32>(0.65, 0.9);
// How far round the limb, in radians, and how far along it, in metres, the
// cloth beside an opening is looked for.
const SEAM_BESIDE_TURN: f32 = 0.25;
const SEAM_BESIDE_REACH: f32 = 0.015;
// Distance from `origin` along `outward` to the outermost face of the
// articles worn beneath, or a negative value where none lies within `limit`.
// Both are in the limb's frame.
fn layer_radius(f: Frame, origin: vec3<f32>, outward: vec3<f32>, limit: f32) -> f32 {
    var outermost = -1.0;
    for (var face = params.body_face_count; face < params.face_count; face++) {
        let a = host_local(f, positions_at(faces[face * 3u]));
        let ab = host_local(f, positions_at(faces[face * 3u + 1u])) - a;
        let ac = host_local(f, positions_at(faces[face * 3u + 2u])) - a;
        let across = cross(outward, ac);
        let facing = dot(ab, across);
        if (facing == 0.0) { continue; }
        let offset = origin - a;
        let u = dot(offset, across) / facing;
        let around = cross(offset, ab);
        let v = dot(outward, around) / facing;
        let reach = dot(ac, around) / facing;
        if (u >= 0.0 && v >= 0.0 && u + v <= 1.0 && reach > outermost && reach <= limit) {
            outermost = reach;
        }
    }
    return outermost;
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
    // How squarely this place faces the body: one toward it, nought away.
    // The body's middle plane lies toward the canonical frame's X origin.
    var turned = 0.0;
    let origin = host_point(f, vec3<f32>(0.0));
    let toward = host_local(f, origin - vec3<f32>(sign(origin.x), 0.0, 0.0)).xz;
    let reach = length(p.xz) * length(toward);
    if (reach > 0.0) { turned = 0.5 + 0.5 * dot(p.xz, toward) / reach; }
    // Share of its length that the head course keeps at this place.
    var kept = 1.0;
    if (params.underarm > 0.0) {
        // A sleeve head ends lower under the arm than over the shoulder: its
        // edge is nearly level over the shoulder and dips sharply under the
        // arm, as a sleeve head is cut.
        let facing = turned * turned;
        let head = smoothstep(params.head, 1.0, axial);
        let drop = params.underarm * max(f.half_extents.x, f.half_extents.z) * facing;
        p.y -= drop * head;
        let course = (1.0 - params.head) * (fitted.y - fitted.x);
        if (course > 0.0) { kept = 1.0 - clamp(drop / course, 0.0, 1.0) * head; }
    }
    let seated = clamp((p.y - fitted.x) / (fitted.y - fitted.x), 0.0, 1.0);
    let station = seated * f32(SECTIONS - 1u);
    let first = min(u32(floor(station)), SECTIONS - 2u);
    let t = station - f32(first);
    let blend = t * t * (3.0 - 2.0 * t);
    let a = first * SECTION_WORDS;
    let b = (first + 1u) * SECTION_WORDS;
    let center = mix(vec2<f32>(sections[a], sections[a + 1u]), vec2<f32>(sections[b], sections[b + 1u]), blend);
    let distance = length(p.xz);
    var direction = vec2<f32>(1.0, 0.0);
    if (distance > 0.0) { direction = p.xz / distance; }
    let ellipse_radius = 1.0 / length(direction / f.half_extents.xz);
    // A shortened course has less slack and a shorter span to throw it
    // over, so its swell falls with the square of the length it keeps. It
    // never falls to nothing: the layers of the cloth must stay apart.
    let allowance = (distance - ellipse_radius) * kept * kept;
    var radius = mix(section_radius(first, direction), section_radius(first + 1u, direction), blend);
    var middle = center;
    if (params.seam > 0.0) {
        // A head set into a seam lies on the cloth it is sewn to. A section
        // is a convex outline, which bridges the hollow between the arm and
        // the body, so near the seam the cloth beneath is found directly.
        // Under the arm the sleeve lies beside the body, not on it.
        let sewn = smoothstep(1.0 - params.seam / (authored.y - authored.x), 1.0, axial)
            * (1.0 - smoothstep(SEAM_BESIDE_BODY.x, SEAM_BESIDE_BODY.y, turned));
        if (sewn > 0.0) {
            let start = vec3<f32>(center.x, p.y, center.y);
            let outward = vec3<f32>(direction.x, 0.0, direction.y);
            let limit = radius + HALF_WIDTH;
            var beneath = layer_radius(f, start, outward, limit);
            if (beneath <= 0.0) {
                // Over an opening in the cloth beneath, the seam lies level
                // with the cloth beside the opening.
                var found = 0.0;
                var level = 0.0;
                for (var side = -1.0; side <= 1.0; side += 2.0) {
                    let turn = side * SEAM_BESIDE_TURN;
                    let turned_out = vec3<f32>(
                        cos(turn) * outward.x - sin(turn) * outward.z, 0.0,
                        sin(turn) * outward.x + cos(turn) * outward.z);
                    let around = layer_radius(f, start, turned_out, limit);
                    if (around > 0.0) { level += around; found += 1.0; }
                    let shifted = start + vec3<f32>(0.0, side * SEAM_BESIDE_REACH, 0.0);
                    let along = layer_radius(f, shifted, outward, limit);
                    if (along > 0.0) { level += along; found += 1.0; }
                }
                if (found > 0.0) { beneath = level / found; }
            }
            if (beneath > 0.0) { radius = mix(radius, min(radius, beneath), sewn); }
        }
    }
    // Below the wrist a cuff keeps its authored length and opens from the
    // wrist section to whichever of it and the hand section is wider.
    let below = min(host_local(f, carriers_at(i)).y - authored.x, 0.0);
    if (below < 0.0 && params.cuff > 0.0) {
        let edge = SECTIONS * SECTION_WORDS;
        let reach = clamp(-below / params.cuff, 0.0, 1.0);
        radius = mix(radius, max(radius, section_radius(SECTIONS, direction)), reach);
        middle = mix(center, vec2<f32>(sections[edge], sections[edge + 1u]), reach);
        p.y += below;
    }
    let radial = middle + direction * (radius + allowance);
    carriers_set(i, host_point(f, vec3<f32>(radial.x, p.y, radial.y)));
}
