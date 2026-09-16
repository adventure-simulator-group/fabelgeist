//! Give cultivated beds exclusive ground triangles inside their packed plot.
use bevy::math::{Vec2, Vec3};

pub(super) fn overlaps(a: [Vec2; 4], b: [Vec2; 4]) -> bool {
    let bounds = |points: [Vec2; 4]| {
        points.into_iter().fold(
            (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
            |(min, max), p| (min.min(p), max.max(p)),
        )
    };
    let (amin, amax) = bounds(a);
    let (bmin, bmax) = bounds(b);
    amin.x < bmax.x && amax.x > bmin.x && amin.y < bmax.y && amax.y > bmin.y
}

pub(super) fn subtract(polygon: Vec<Vec3>, beds: &[[Vec2; 4]]) -> Vec<Vec<Vec3>> {
    let mut pieces = vec![polygon];
    for bed in beds {
        pieces = pieces
            .into_iter()
            .flat_map(|piece| outside(piece, *bed))
            .collect();
    }
    pieces
}

fn outside(mut remaining: Vec<Vec3>, bed: [Vec2; 4]) -> Vec<Vec<Vec3>> {
    let winding = (bed[1] - bed[0]).perp_dot(bed[3] - bed[0]).signum();
    let mut pieces = Vec::new();
    for side in 0..4 {
        let start = bed[side];
        let edge = bed[(side + 1) % 4] - start;
        let distance = |p: Vec3| edge.perp_dot(Vec2::new(p.x, p.z) - start) * winding;
        let mut inside = Vec::new();
        let mut outside = Vec::new();
        for index in 0..remaining.len() {
            let a = remaining[index];
            let b = remaining[(index + 1) % remaining.len()];
            let da = distance(a);
            let db = distance(b);
            if da >= 0.0 {
                inside.push(a);
            }
            if da <= 0.0 {
                outside.push(a);
            }
            if (da > 0.0 && db < 0.0) || (da < 0.0 && db > 0.0) {
                let crossing = a.lerp(b, da / (da - db));
                inside.push(crossing);
                outside.push(crossing);
            }
        }
        if outside.len() >= 3 {
            pieces.push(outside);
        }
        remaining = inside;
        if remaining.len() < 3 {
            break;
        }
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bed_removal_preserves_sloped_ground_and_exact_working_area() {
        let point = |x, z| Vec3::new(x, x * 0.2 + z * 0.1, z);
        let plot = vec![
            point(-3.0, -2.0),
            point(3.0, -2.0),
            point(3.0, 2.0),
            point(-3.0, 2.0),
        ];
        let bed = [
            Vec2::new(-2.0, -1.0),
            Vec2::new(0.0, -1.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(-2.0, 1.0),
        ];
        for bed in [bed, [bed[3], bed[2], bed[1], bed[0]]] {
            let pieces = subtract(plot.clone(), &[bed]);
            let mut area = 0.0;
            for polygon in pieces {
                for p in &polygon {
                    assert!((p.y - p.x * 0.2 - p.z * 0.1).abs() < 0.0001);
                }
                for index in 1..polygon.len() - 1 {
                    let [a, b, c] = [polygon[0], polygon[index], polygon[index + 1]];
                    area += ((b.x - a.x) * (c.z - a.z) - (b.z - a.z) * (c.x - a.x)).abs() * 0.5;
                    let centre = (a + b + c) / 3.0;
                    assert!(
                        centre.x <= -2.0 || centre.x >= 0.0 || centre.z <= -1.0 || centre.z >= 1.0
                    );
                }
            }
            assert!((area - 20.0).abs() < 0.0001);
        }
    }
}
