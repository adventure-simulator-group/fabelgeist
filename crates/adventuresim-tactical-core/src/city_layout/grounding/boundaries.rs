//! Exterior segments of an owned union retain its exact declared half-planes.
use bevy::math::DVec2;

pub(super) fn exterior(regions: &[Vec<DVec2>]) -> Vec<[DVec2; 2]> {
    let mut result = Vec::new();
    for (owner, corners) in regions.iter().enumerate() {
        for i in 0..corners.len() {
            let a = corners[i];
            let b = corners[(i + 1) % corners.len()];
            let direction = b - a;
            let mut stations = vec![0.0, 1.0];
            for region in regions {
                for j in 0..region.len() {
                    let origin = region[j];
                    let edge = region[(j + 1) % region.len()] - origin;
                    // A shared endpoint can round just beyond the segment's
                    // closed intersection interval. Retain projected corners
                    // on this same half-plane before testing a crossing.
                    for point in [origin, origin + edge] {
                        if direction.perp_dot(point - a).abs()
                            <= predicate_error(direction, point, a)
                        {
                            let t = (point - a).dot(direction) / direction.length_squared();
                            if (0.0..=1.0).contains(&t) {
                                stations.push(t);
                            }
                        }
                    }
                    let divisor = direction.perp_dot(edge);
                    if divisor.abs() > f64::EPSILON {
                        let t = (origin - a).perp_dot(edge) / divisor;
                        let u = (origin - a).perp_dot(direction) / divisor;
                        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
                            stations.push(t);
                        }
                    }
                }
            }
            stations.sort_by(f64::total_cmp);
            stations.dedup();
            for pair in stations.windows(2) {
                let midpoint = a + direction * ((pair[0] + pair[1]) * 0.5);
                let outward = -direction.perp();
                let covered = regions.iter().enumerate().any(|(other, region)| {
                    if other == owner {
                        return false;
                    }
                    let mut boundary = false;
                    let mut exits = false;
                    for j in 0..region.len() {
                        let edge = region[(j + 1) % region.len()] - region[j];
                        let side = edge.perp_dot(midpoint - region[j]);
                        let error = predicate_error(edge, midpoint, region[j]);
                        if side < -error {
                            return false;
                        }
                        if side.abs() <= error {
                            boundary = true;
                            exits |= edge.perp_dot(outward) < 0.0;
                        }
                    }
                    !exits || (boundary && other < owner)
                });
                if !covered && pair[1] > pair[0] {
                    result.push([a + direction * pair[0], a + direction * pair[1]]);
                }
            }
        }
    }
    result
}

fn predicate_error(edge: DVec2, point: DVec2, origin: DVec2) -> f64 {
    // Bound eight scalar operations, including scene-origin subtraction.
    // This is arithmetic roundoff, not a grading or contact envelope.
    f64::EPSILON * edge.length() * (point.length() + origin.length()).max(1.0) * 8.0
}

pub(super) fn segment_interval(
    segment: [DVec2; 2],
    polygon: &[DVec2],
    tolerance_metres: f64,
) -> Option<(f64, f64)> {
    let [a, b] = segment;
    let direction = b - a;
    let mut begin = 0.0_f64;
    let mut end = 1.0_f64;
    for i in 0..polygon.len() {
        let edge = polygon[(i + 1) % polygon.len()] - polygon[i];
        let origin = edge.perp_dot(a - polygon[i]) + tolerance_metres * edge.length();
        let rate = edge.perp_dot(direction);
        if rate.abs() <= f64::EPSILON {
            if origin < 0.0 {
                return None;
            }
        } else if rate > 0.0 {
            begin = begin.max(-origin / rate);
        } else {
            end = end.min(-origin / rate);
        }
    }
    (end > begin).then_some((begin, end))
}

#[cfg(test)]
mod tests;
