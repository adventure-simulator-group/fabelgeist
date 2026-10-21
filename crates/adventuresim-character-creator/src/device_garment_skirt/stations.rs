//! Supported envelope points and nearest-height sampling for skirt cages.

/// One invocation per drape cage station: the support of the section's points
/// in each direction, then the radius each direction must reach to enclose
/// every supporting line.
pub(super) const DRAPE_STATIONS: &str = r#"
const SUPPORT_MASK: u32 = 3u;
const SECTION_HALF_WIDTH_M: f32 = 0.018;
const MINIMUM_SECTION_POINTS: u32 = 12u;

var<private> supports: array<f32, SEGMENTS>;

// The envelope: supported skin in vertex order, then each leg's garment.
fn envelope_count() -> u32 {
    return arrayLength(&support) + params.left + params.right;
}

struct EnvelopePoint {
    present: bool,
    position: vec3<f32>,
};

// Unsupported skin has no envelope payload.
fn envelope(slot: u32, f: Frame) -> EnvelopePoint {
    let skin = arrayLength(&support);
    var p: vec3<f32>;
    if (slot < skin) {
        if ((support[slot] & SUPPORT_MASK) == 0u) {
            return EnvelopePoint(false, vec3<f32>());
        }
        p = positions_at(slot);
    } else if (slot < skin + params.left) {
        p = points_at(slot - skin);
    } else {
        p = extra_at(slot - skin - params.left);
    }
    return EnvelopePoint(true, host_local(f, p));
}

fn include(p: vec3<f32>) {
    for (var col = 0u; col < SEGMENTS; col = col + 1u) {
        let angle = TAU * f32(col) / f32(SEGMENTS);
        supports[col] = max(supports[col], p.x * sin(angle) + p.z * cos(angle));
    }
}

@compute @workgroup_size(1)
fn main(@builtin(workgroup_id) group: vec3<u32>) {
    let row = group.x;
    let f = frame_at(0u);
    let y = f.half_extents.y * (2.0 * f32(row) / f32(STATIONS - 1u) - 1.0);
    for (var col = 0u; col < SEGMENTS; col = col + 1u) {
        supports[col] = 0.0;
    }
    let count = envelope_count();
    var near = 0u;
    var total = 0u;
    for (var slot = 0u; slot < count; slot = slot + 1u) {
        let p = envelope(slot, f);
        if (!p.present) {
            continue;
        }
        total = total + 1u;
        if (abs(p.position.y - y) < SECTION_HALF_WIDTH_M) {
            near = near + 1u;
            include(p.position);
        }
    }
    if (near < MINIMUM_SECTION_POINTS && near < total) {
        // The nearest points by height instead, ties to the earliest.
        for (var col = 0u; col < SEGMENTS; col = col + 1u) {
            supports[col] = 0.0;
        }
        var cursor_present = false;
        var last_distance: f32;
        var last: u32;
        for (var k = 0u; k < min(MINIMUM_SECTION_POINTS, total); k = k + 1u) {
            var candidate_present = false;
            var best_distance: f32;
            var best: u32;
            var best_point: vec3<f32>;
            for (var slot = 0u; slot < count; slot = slot + 1u) {
                let p = envelope(slot, f);
                if (!p.present) {
                    continue;
                }
                let d = abs(p.position.y - y);
                if (!(d <= INFINITY)) {
                    continue;
                }
                let after = !cursor_present || d > last_distance || (d == last_distance && slot > last);
                let better = !candidate_present || d < best_distance || (d == best_distance && slot < best);
                if (after && better) {
                    candidate_present = true;
                    best_distance = d;
                    best = slot;
                    best_point = p.position;
                }
            }
            if (!candidate_present) {
                break;
            }
            include(best_point);
            cursor_present = true;
            last_distance = best_distance;
            last = best;
        }
        if (!cursor_present) {
            atomicOr(&candidate_status[0], 1u);
            return;
        }
    }
    if (total == 0u) {
        atomicOr(&candidate_status[0], 1u);
        return;
    }
    for (var col = 0u; col < SEGMENTS; col = col + 1u) {
        let angle = TAU * f32(col) / f32(SEGMENTS);
        var radius = INFINITY;
        for (var normal = 0u; normal < SEGMENTS; normal = normal + 1u) {
            let normal_angle = TAU * f32(normal) / f32(SEGMENTS);
            let facing = cos(angle - normal_angle);
            if (facing > 0.0) {
                radius = min(radius, supports[normal] / facing);
            }
        }
        fit[CAGE + row * SEGMENTS + col] = radius;
    }
}
"#;
