//! Distinguish intended welded adjacency from overlap beyond that feature.
use fabelgeist_math::Vec3;

use super::{CONTACT_DISTANCE_TOLERANCE_METERS as TOLERANCE, cross, dot};

type Point = [f64; 3];

pub(super) fn beyond_shared_feature(a: [Vec3; 3], b: [Vec3; 3]) -> bool {
    let shared = a.into_iter().filter(|p| b.contains(p)).collect::<Vec<_>>();
    if shared.is_empty() || shared.len() == 3 {
        return true;
    }
    let a = a.map(|p| p.to_array().map(f64::from));
    let b = b.map(|p| p.to_array().map(f64::from));
    let normals = [normal(a), normal(b)];
    if b.iter()
        .all(|&p| dot(sub(p, a[0]), normals[0]).abs() <= TOLERANCE)
    {
        return coplanar_contact_beyond(a, b, normals[0], &shared);
    }
    // Two noncoplanar triangles sharing an edge meet only along that edge.
    if shared.len() == 2 {
        return false;
    }
    let line = unit(cross(normals[0], normals[1]));
    let Some(ai) = section(a, b[0], normals[1], line) else {
        return false;
    };
    let Some(bi) = section(b, a[0], normals[0], line) else {
        return false;
    };
    let contact = (ai.0.max(bi.0), ai.1.min(bi.1));
    if contact.0 > contact.1 + TOLERANCE {
        return false;
    }
    let allowed = dot(shared[0].to_array().map(f64::from), line);
    contact.0 < allowed - TOLERANCE || contact.1 > allowed + TOLERANCE
}

fn sub(a: Point, b: Point) -> Point {
    std::array::from_fn(|k| a[k] - b[k])
}

fn unit(p: Point) -> Point {
    let length = dot(p, p).sqrt();
    p.map(|v| v / length)
}

fn normal(p: [Point; 3]) -> Point {
    unit(cross(sub(p[1], p[0]), sub(p[2], p[0])))
}

fn section(p: [Point; 3], origin: Point, normal: Point, line: Point) -> Option<(f64, f64)> {
    let distances = p.map(|p| dot(sub(p, origin), normal));
    let mut interval: Option<(f64, f64)> = None;
    let mut include = |point| {
        let value = dot(point, line);
        interval = Some(match interval {
            Some((low, high)) => (low.min(value), high.max(value)),
            None => (value, value),
        });
    };
    for i in 0..3 {
        if distances[i].abs() <= TOLERANCE {
            include(p[i]);
        }
        let next = (i + 1) % 3;
        if distances[i] * distances[next] < 0.0 {
            let t = distances[i] / (distances[i] - distances[next]);
            include(std::array::from_fn(|k| {
                p[i][k] + t * (p[next][k] - p[i][k])
            }));
        }
    }
    interval
}

fn coplanar_contact_beyond(a: [Point; 3], b: [Point; 3], normal: Point, shared: &[Vec3]) -> bool {
    let allowed = |point: Point| {
        let first = shared[0].to_array().map(f64::from);
        let closest = if shared.len() == 1 {
            first
        } else {
            let edge = sub(shared[1].to_array().map(f64::from), first);
            let t = (dot(sub(point, first), edge) / dot(edge, edge)).clamp(0.0, 1.0);
            std::array::from_fn(|k| first[k] + t * edge[k])
        };
        let distance = sub(point, closest);
        dot(distance, distance) <= TOLERANCE * TOLERANCE
    };
    let inside = |point: Point, triangle: [Point; 3]| {
        let sides = std::array::from_fn::<_, 3, _>(|i| {
            let edge = sub(triangle[(i + 1) % 3], triangle[i]);
            dot(cross(edge, sub(point, triangle[i])), normal) / dot(edge, edge).sqrt()
        });
        sides.iter().all(|&d| d >= -TOLERANCE) || sides.iter().all(|&d| d <= TOLERANCE)
    };
    if a.into_iter().any(|p| inside(p, b) && !allowed(p))
        || b.into_iter().any(|p| inside(p, a) && !allowed(p))
    {
        return true;
    }
    // Vertices and edge crossings are the vertices of the convex intersection,
    // including zero-area segment contacts. Each must lie on the shared feature.
    for i in 0..3 {
        let ae = sub(a[(i + 1) % 3], a[i]);
        for j in 0..3 {
            let be = sub(b[(j + 1) % 3], b[j]);
            let denominator = dot(cross(ae, be), normal);
            if denominator == 0.0 {
                continue;
            }
            let delta = sub(b[j], a[i]);
            let t = dot(cross(delta, be), normal) / denominator;
            let u = dot(cross(delta, ae), normal) / denominator;
            if (0.0..=1.0).contains(&t)
                && (0.0..=1.0).contains(&u)
                && !allowed(std::array::from_fn(|k| a[i][k] + t * ae[k]))
            {
                return true;
            }
        }
    }
    false
}

#[test]
fn adjacency_does_not_exempt_coplanar_overlap_or_crossing_past_a_shared_vertex() {
    let a = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]].map(Vec3::from_array);
    let beyond = [[0.0, 0.0, 0.0], [0.5, 0.5, -1.0], [0.5, 0.5, 1.0]].map(Vec3::from_array);
    assert!(beyond_shared_feature(a, beyond));
    let touch = [[0.0, 0.0, 0.0], [-1.0, 0.0, 1.0], [0.0, -1.0, 1.0]].map(Vec3::from_array);
    assert!(!beyond_shared_feature(a, touch));
    let overlap = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.5, 0.0]].map(Vec3::from_array);
    assert!(beyond_shared_feature(a, overlap));
    let edge = [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, -1.0, 0.0]].map(Vec3::from_array);
    assert!(!beyond_shared_feature(a, edge));
    assert!(beyond_shared_feature(a, a));
    let segment = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, -1.0, 0.0]].map(Vec3::from_array);
    assert!(beyond_shared_feature(a, segment));
}
