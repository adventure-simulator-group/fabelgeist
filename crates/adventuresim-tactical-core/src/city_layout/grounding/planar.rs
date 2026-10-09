//! Exact convex half-plane arithmetic shared by support and access proofs.
pub(super) mod query;
pub(super) fn clip(
    polygon: Vec<bevy::math::DVec2>,
    side: impl Fn(bevy::math::DVec2) -> f64,
) -> Vec<bevy::math::DVec2> {
    let mut result = Vec::new();
    let Some(mut previous) = polygon.last().copied() else {
        return result;
    };
    let mut previous_side = side(previous);
    for point in polygon {
        let current_side = side(point);
        if (current_side >= 0.0) != (previous_side >= 0.0) {
            result.push(previous.lerp(point, previous_side / (previous_side - current_side)));
        }
        if current_side >= 0.0 {
            result.push(point);
        }
        previous = point;
        previous_side = current_side;
    }
    result
}

pub(super) fn subtract(
    mut inside: Vec<bevy::math::DVec2>,
    corners: &[bevy::math::DVec2],
) -> Vec<Vec<bevy::math::DVec2>> {
    // Half-plane clipping can partition a wholly outside triangle along the
    // infinite extensions of property edges. Test the complete intersection
    // first: unrelated source triangles must keep their original topology.
    let mut intersection = inside.clone();
    for index in 0..corners.len() {
        let a = corners[index];
        let edge = corners[(index + 1) % corners.len()] - a;
        intersection = clip(intersection, |p| edge.perp_dot(p - a));
    }
    if intersection.len() < 3 || signed_area(&intersection).abs() <= f64::EPSILON {
        return vec![inside];
    }
    let mut outside = Vec::new();
    for index in 0..corners.len() {
        let a = corners[index];
        let edge = corners[(index + 1) % corners.len()] - a;
        let remainder = clip(inside.clone(), |p| -edge.perp_dot(p - a));
        if remainder.len() >= 3 {
            outside.push(remainder);
        }
        inside = clip(inside, |p| edge.perp_dot(p - a));
    }
    outside
}

pub(super) fn signed_area(polygon: &[bevy::math::DVec2]) -> f64 {
    let Some(origin) = polygon.first() else {
        return 0.0;
    };
    polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
        .map(|(a, b)| (*a - *origin).perp_dot(*b - *origin))
        .sum::<f64>()
        * 0.5
}

/// Distance to the complete convex outline; an interior point is supported.
pub(super) fn distance_outside(point: bevy::math::DVec2, outline: &[bevy::math::DVec2]) -> f64 {
    if (0..outline.len()).all(|i| {
        (outline[(i + 1) % outline.len()] - outline[i]).perp_dot(point - outline[i]) >= 0.0
    }) {
        return 0.0;
    }
    (0..outline.len())
        .map(|i| {
            let a = outline[i];
            let edge = outline[(i + 1) % outline.len()] - a;
            let fraction = ((point - a).dot(edge) / edge.length_squared()).clamp(0.0, 1.0);
            point.distance(a + edge * fraction)
        })
        .fold(f64::INFINITY, f64::min)
}
