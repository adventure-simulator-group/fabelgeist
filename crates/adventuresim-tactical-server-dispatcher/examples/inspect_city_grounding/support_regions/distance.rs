//! Distance between convex plan-space contacts, including point/edge contacts.
use bevy::math::Vec2;

pub(super) fn between(a: &[Vec2], b: &[Vec2]) -> Option<f32> {
    if a.is_empty() || b.is_empty() {
        return None;
    }
    if a.iter().any(|p| contains(b, *p)) || b.iter().any(|p| contains(a, *p)) {
        return Some(0.0);
    }
    let mut minimum = f32::INFINITY;
    for (&start, &end) in a.iter().zip(a.iter().cycle().skip(1)).take(a.len()) {
        for (&other_start, &other_end) in b.iter().zip(b.iter().cycle().skip(1)).take(b.len()) {
            if crosses(start, end, other_start, other_end) {
                return Some(0.0);
            }
            for distance in [
                to_segment(start, other_start, other_end),
                to_segment(end, other_start, other_end),
                to_segment(other_start, start, end),
                to_segment(other_end, start, end),
            ] {
                minimum = minimum.min(distance);
            }
        }
    }
    Some(minimum)
}

fn to_segment(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let edge = end - start;
    if edge.length_squared() == 0.0 {
        return point.distance(start);
    }
    let fraction = ((point - start).dot(edge) / edge.length_squared()).clamp(0.0, 1.0);
    point.distance(start + edge * fraction)
}

fn crosses(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    let r = b - a;
    let s = d - c;
    let denominator = r.perp_dot(s);
    if denominator == 0.0 {
        return false;
    }
    let t = (c - a).perp_dot(s) / denominator;
    let u = (c - a).perp_dot(r) / denominator;
    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

fn contains(polygon: &[Vec2], point: Vec2) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let mut positive = false;
    let mut negative = false;
    for (&start, &end) in polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
    {
        let side = (end - start).perp_dot(point - start);
        positive |= side > 0.0;
        negative |= side < 0.0;
    }
    !(positive && negative)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crossed_edges_touching_edges_and_separated_contacts_are_distinguished() {
        let a = [
            Vec2::new(-2.0, -0.1),
            Vec2::new(2.0, -0.1),
            Vec2::new(2.0, 0.1),
            Vec2::new(-2.0, 0.1),
        ];
        let b = a.map(|p| Vec2::new(-p.y, p.x));
        assert_eq!(between(&a, &b), Some(0.0));
        let touching = a.map(|p| p + Vec2::Y * 0.2);
        assert_eq!(between(&a, &touching), Some(0.0));
        let separated = a.map(|p| p + Vec2::Y);
        assert!((between(&a, &separated).unwrap() - 0.8).abs() < 0.00001);
        assert_eq!(between(&a, &separated), between(&separated, &a));
        assert_eq!(between(&[], &a), None);
    }
}
