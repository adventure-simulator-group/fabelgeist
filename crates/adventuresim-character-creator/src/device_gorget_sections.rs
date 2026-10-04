//! Gorget measurements from triangle sections, independent of vertex density.

/// Each invocation intersects a horizontal body section, clipped to the
/// sagittal strip for front/back depths. A coarse body need not have vertices
/// inside that strip or at the measurement height.
pub(crate) const BANDS: &str = r#"
@compute @workgroup_size(1)
fn main(@builtin(workgroup_id) group: vec3<u32>) {
    let band = group.x;
    let height = fit[HEIGHT];
    let lower = plane_section(1u);
    var y = height * params.shoulder_ratio;
    var half_width = INFINITY;
    if (band > 0u) {
        let t = f32((band - 1u) % 3u) * 0.5;
        half_width = height * params.sagittal_ratio;
        if (band < 4u) {
            let start = plane_at(1u, lower.high.z);
            y = start + (fit[HEM] - start) * t;
        } else {
            let start = plane_at(1u, lower.low.z);
            y = start + (fit[HEM + 2u] - start) * t;
        }
    }
    var low = vec3<f32>(INFINITY);
    var high = vec3<f32>(-INFINITY);
    for (var face = 0u; face < arrayLength(&faces) / 3u; face += 1u) {
        var section: array<vec3<f32>, 2>;
        var count = 0u;
        for (var edge = 0u; edge < 3u; edge += 1u) {
            let a = points_at(faces[face * 3u + edge]);
            let b = points_at(faces[face * 3u + (edge + 1u) % 3u]);
            if ((a.y > y) != (b.y > y)) {
                section[count] = a + (b - a) * ((y - a.y) / (b.y - a.y));
                count += 1u;
            }
        }
        if (count != 2u) { continue; }
        let a = section[0];
        let delta = section[1] - a;
        var first = 0.0;
        var last = 1.0;
        if (band > 0u) {
            if (delta.x == 0.0) {
                if (abs(a.x) > half_width) { continue; }
            } else {
                let left = (-half_width - a.x) / delta.x;
                let right = (half_width - a.x) / delta.x;
                first = max(first, min(left, right));
                last = min(last, max(left, right));
                if (first > last) { continue; }
            }
        }
        let start = a + delta * first;
        let end = a + delta * last;
        low = min(low, min(start, end));
        high = max(high, max(start, end));
    }
    let at = BANDS + band * 6u;
    for (var axis = 0u; axis < 3u; axis += 1u) {
        fit[at + axis] = low[axis];
        fit[at + 3u + axis] = high[axis];
    }
}
"#;
